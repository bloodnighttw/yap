use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming, StatusCode, Method};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use http_body_util::{Empty, Full, BodyExt};
use hyper::body::Bytes;
use chrono::{DateTime, Utc};

use super::Component;
use crate::{config::Config, framework::Updater};

#[derive(Clone, Debug)]
pub struct HttpLog {
    pub method: String,
    pub uri: String,
    pub timestamp: DateTime<Utc>,
}

pub type SharedLogs = Arc<RwLock<VecDeque<HttpLog>>>;

#[derive(Clone)]
pub struct Proxy {
    logs: SharedLogs,
    updater: Option<Updater>,
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(10))),
            updater: None,
        }
    }
}

impl Proxy {
    pub fn get_logs(&self) -> SharedLogs {
        self.logs.clone()
    }

    async fn log_request(
        method: &str,
        uri: &str,
        logs: SharedLogs,
        updater: &Option<Updater>,
    ) {
        let timestamp = Utc::now();
        
        // Store the log
        {
            let mut logs_guard = logs.write().await;
            if logs_guard.len() >= 10 {
                logs_guard.pop_front();
            }
            logs_guard.push_back(HttpLog {
                method: method.to_string(),
                uri: uri.to_string(),
                timestamp,
            });
        }

        // Trigger UI update
        if let Some(updater) = updater {
            let _ = updater.update();
        }
    }

    async fn handle_connect(
        req: Request<Incoming>,
        mut stream: tokio::net::TcpStream,
        logs: SharedLogs,
        updater: Option<Updater>,
    ) {
        let uri = req.uri().to_string();
        info!("CONNECT to {}", uri);
        
        // Log the CONNECT request
        Self::log_request("CONNECT", &uri, logs, &updater).await;

        // Parse the host and port
        let host_port = uri.split(':').collect::<Vec<&str>>();
        if host_port.len() != 2 {
            error!("Invalid CONNECT URI: {}", uri);
            return;
        }

        let host = host_port[0];
        let port = host_port[1];

        // Connect to the target server
        let target_addr = format!("{}:{}", host, port);
        let mut target_stream = match tokio::net::TcpStream::connect(&target_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to {}: {}", target_addr, e);
                let _ = stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
                return;
            }
        };

        // Send 200 Connection Established
        if let Err(e) = stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await {
            error!("Failed to send CONNECT response: {}", e);
            return;
        }

        // Tunnel data between client and target
        let (mut client_read, mut client_write) = stream.split();
        let (mut target_read, mut target_write) = target_stream.split();

        let client_to_target = tokio::io::copy(&mut client_read, &mut target_write);
        let target_to_client = tokio::io::copy(&mut target_read, &mut client_write);

        match tokio::try_join!(client_to_target, target_to_client) {
            Ok((bytes_to_target, bytes_to_client)) => {
                info!("Tunnel closed: {} bytes to target, {} bytes to client", bytes_to_target, bytes_to_client);
            }
            Err(e) => {
                warn!("Tunnel error: {}", e);
            }
        }
    }

    async fn handle_request(
        req: Request<Incoming>,
        logs: SharedLogs,
        updater: Option<Updater>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        info!("Received {} {}", method, uri);

        // Log the request
        Self::log_request(method.as_str(), &uri.to_string(), logs.clone(), &updater).await;

        // For regular HTTP requests (not CONNECT), forward them
        if method != Method::CONNECT {
            // Build the client request
            let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

            match client.request(req).await {
                Ok(response) => {
                    let status = response.status();
                    let headers = response.headers().clone();
                    
                    // Read the body
                    let body_bytes = match response.into_body().collect().await {
                        Ok(collected) => collected.to_bytes(),
                        Err(e) => {
                            error!("Failed to read response body: {}", e);
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_GATEWAY)
                                .body(Full::new(Bytes::from("Failed to read response")))
                                .unwrap());
                        }
                    };

                    let mut resp = Response::builder()
                        .status(status);
                    
                    // Copy headers
                    for (name, value) in headers.iter() {
                        resp = resp.header(name, value);
                    }

                    return Ok(resp.body(Full::new(body_bytes)).unwrap());
                }
                Err(e) => {
                    error!("Failed to forward request: {}", e);
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Full::new(Bytes::from(format!("Failed to forward request: {}", e))))
                        .unwrap());
                }
            }
        }

        // For CONNECT, return OK (shouldn't reach here as CONNECT is handled separately)
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::new()))
            .unwrap())
    }

    async fn run_server(logs: SharedLogs, updater: Option<Updater>) {
        let addr = SocketAddr::from(([127, 0, 0, 1], 9999));
        
        let listener = match TcpListener::bind(addr).await {
            Ok(listener) => {
                info!("Proxy server listening on {}", addr);
                listener
            }
            Err(e) => {
                error!("Failed to bind to {}: {}", addr, e);
                return;
            }
        };

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let logs = logs.clone();
            let updater = updater.clone();

            tokio::spawn(async move {
                // Peek at the first request to see if it's CONNECT
                let io = TokioIo::new(stream);
                
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let logs = logs.clone();
                            let updater = updater.clone();
                            async move {
                                if req.method() == Method::CONNECT {
                                    // For CONNECT, we need to hijack the connection
                                    // Return a special response that won't be sent
                                    // This is a limitation - we'll handle it differently
                                    Ok::<_, hyper::Error>(Response::builder()
                                        .status(StatusCode::OK)
                                        .body(Full::new(Bytes::new()))
                                        .unwrap())
                                } else {
                                    Self::handle_request(req, logs, updater).await
                                }
                            }
                        }),
                    )
                    .with_upgrades()
                    .await
                {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

impl Component for Proxy {
    fn component_will_mount(&mut self, _config: Config) -> color_eyre::Result<()> {
        info!("Proxy::component_will_mount - Initializing proxy");
        Ok(())
    }

    fn component_did_mount(
        &mut self,
        _area: ratatui::layout::Size,
        updater: Updater,
    ) -> color_eyre::Result<()> {
        info!("Proxy::component_did_mount - Starting proxy server");
        self.updater = Some(updater.clone());
        
        let logs = self.logs.clone();
        let updater_clone = Some(updater);
        
        tokio::spawn(async move {
            Self::run_server(logs, updater_clone).await;
        });
        
        Ok(())
    }

    fn render(
        &mut self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        // This component doesn't render anything itself
        Ok(())
    }
}
