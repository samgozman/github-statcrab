mod cards;
mod web;

use axum::serve;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = web::app_router();

    // Bind address (0.0.0.0 to be accessible in containers; localhost otherwise)
    let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid socket address");
    let listener = TcpListener::bind(addr).await.expect("bind tcp listener");
    println!("Listening on http://{}", listener.local_addr().unwrap());

    if let Err(e) = serve(listener, app).await {
        eprintln!("server error: {e}");
    }
}
