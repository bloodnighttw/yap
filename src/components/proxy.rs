use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{info, error};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming, StatusCode, Method};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use http_body_util::{Full, BodyExt};
use hyper::body::Bytes;
use chrono::{DateTime, Utc};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

use super::Component;
use crate::{config::Config, framework::Updater};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct HttpLog {
    pub method: String,
    pub uri: String,
    pub timestamp: DateTime<Utc>,
    pub path: String,
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
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            updater: None,
        }
    }
}

#[allow(dead_code)]
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
            let id = uri.to_string();
            if logs_guard.len() >= 100 {
                logs_guard.pop_front();
            }
            logs_guard.push_back(HttpLog {
                method: method.to_string(),
                uri: uri.to_string(),
                timestamp,
                path: id,
            });
        }

        // Write to file
        if let Err(e) = Self::write_log_to_file(method, uri, timestamp).await {
            error!("Failed to write log to file: {}", e);
        }

        // Trigger UI update
        if let Some(updater) = updater {
            let _ = updater.update();
        }
    }

    async fn write_log_to_file(
        method: &str,
        uri: &str,
        timestamp: DateTime<Utc>,
    ) -> std::io::Result<()> {
        let log_line = format!(
            "{} {} {}\n",
            timestamp.to_rfc3339(),
            method,
            uri
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("proxy_requests.log")
            .await?;

        file.write_all(log_line.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    fn uri_to_file_path(uri: &str) -> PathBuf {
        // Parse the URI to extract hostname and path
        let parsed = match url::Url::parse(uri) {
            Ok(url) => url,
            Err(_) => {
                // If parsing fails, create a safe filename from the raw URI
                let safe_name = uri.replace(['/', ':', '?', '&', '='], "_");
                return PathBuf::from(".yap").join("unknown").join(format!("{}.yap", safe_name));
            }
        };

        let host = parsed.host_str().unwrap_or("unknown");
        let path = parsed.path();
        
        // Create the base directory structure
        let mut file_path = PathBuf::from(".yap").join(host);
        
        // Convert path to filesystem-safe structure
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        if path_parts.is_empty() {
            // Root path
            file_path.push("index");
        } else {
            for part in path_parts {
                // Sanitize each part to be filesystem-safe
                let safe_part = part.replace([':', '?', '&', '=', '*', '<', '>', '|', '"'], "_");
                file_path.push(safe_part);
            }
        }
        
        // Add query parameters to the filename if present
        if let Some(query) = parsed.query() {
            let query_safe = query.replace(['/', ':', '?', '&', '=', '*', '<', '>', '|', '"'], "_");
            let current_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            file_path.set_file_name(format!("{}_{}", current_name, query_safe));
        }
        
        // Add .yap extension
        let final_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        file_path.set_file_name(format!("{}.yap", final_name));
        
        file_path
    }

    fn is_binary_content(content_type: Option<&str>) -> bool {
        if let Some(ct) = content_type {
            let ct_lower = ct.to_lowercase();
            ct_lower.starts_with("image/")
                || ct_lower.starts_with("video/")
                || ct_lower.starts_with("audio/")
                || ct_lower.starts_with("application/octet-stream")
                || ct_lower.starts_with("application/pdf")
                || ct_lower.starts_with("application/zip")
                || ct_lower.starts_with("font/")
        } else {
            false
        }
    }

    async fn save_request_to_file(
        method: &str,
        uri: &str,
        _headers: &hyper::HeaderMap,
        _body: Option<&Bytes>,
        response_status: u16,
        response_headers: &hyper::HeaderMap,
        response_body: &Bytes,
        timestamp: DateTime<Utc>,
    ) -> std::io::Result<()> {
        let file_path = Self::uri_to_file_path(uri);
        
        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Get content type
        let content_type = response_headers
            .get("content-type")
            .and_then(|v| v.to_str().ok());
        
        let is_binary = Self::is_binary_content(content_type);
        
        // Create the log content
        let mut content = String::new();
        content.push_str("=== HTTP Response ===\n");
        content.push_str(&format!("Timestamp: {}\n", timestamp.to_rfc3339()));
        content.push_str(&format!("Method: {}\n", method));
        content.push_str(&format!("URI: {}\n", uri));
        content.push_str(&format!("Status: {}\n\n", response_status));
        
        content.push_str("Response Headers:\n");
        for (name, value) in response_headers.iter() {
            if let Ok(value_str) = value.to_str() {
                content.push_str(&format!("  {}: {}\n", name, value_str));
            }
        }
        content.push_str("\n");
        
        if is_binary {
            // Save binary data to a separate file
            let binary_file_path = file_path.with_extension("bin");
            let mut binary_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&binary_file_path)
                .await?;
            
            binary_file.write_all(response_body).await?;
            binary_file.flush().await?;
            
            content.push_str("Response Body:\n");
            content.push_str(&format!("[Binary data stored in: {}]\n", binary_file_path.display()));
            content.push_str(&format!("Size: {} bytes\n", response_body.len()));
            
            info!("Saved binary data to: {}", binary_file_path.display());
        } else {
            content.push_str("Response Body:\n");
            if response_body.is_empty() {
                content.push_str("[Empty]\n");
            } else {
                content.push_str(&String::from_utf8_lossy(response_body));
            }
        }
        
        // Write log to file
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .await?;
        
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;
        
        info!("Saved request to: {}", file_path.display());
        
        Ok(())
    }

    async fn handle_request(
        req: Request<Incoming>,
        logs: SharedLogs,
        updater: Option<Updater>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let req_headers = req.headers().clone();
        let timestamp = Utc::now();
        
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

                    // Save the request and response to file (without request body for now)
                    if let Err(e) = Self::save_request_to_file(
                        method.as_str(),
                        &uri.to_string(),
                        &req_headers,
                        None,  // We don't save request body to avoid consuming the stream
                        status.as_u16(),
                        &headers,
                        &body_bytes,
                        timestamp,
                    ).await {
                        error!("Failed to save request to file: {}", e);
                    }

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
