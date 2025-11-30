use crate::components::http_proxy::HttpRequest;
use chrono::Local;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

/// Start HTTP proxy server
pub async fn start_proxy_server(tx: mpsc::UnboundedSender<HttpRequest>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    eprintln!("Proxy server listening on http://{}", addr);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };

        let io = TokioIo::new(stream);
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| handle_request(req, tx.clone())),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    tx: mpsc::UnboundedSender<HttpRequest>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let timestamp = Local::now().format("%H:%M:%S").to_string();

    // Send to TUI component
    let _ = tx.send(HttpRequest {
        method: method.clone(),
        uri: uri.clone(),
        timestamp,
    });

    // Simple echo response
    let response_body = format!(
        "HTTP Proxy received:\nMethod: {}\nURI: {}\n",
        method, uri
    );

    Ok(Response::new(Full::new(Bytes::from(response_body))))
}
