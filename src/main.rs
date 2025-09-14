mod cards;
mod github;
mod web;

use anyhow::Result;
use std::net::SocketAddr;

fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();

    let app = web::app_router();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            // Bind address (0.0.0.0 to be accessible in containers; localhost otherwise)
            let addr: SocketAddr = "0.0.0.0:3000".parse().expect("valid socket address");

            // Create the TCP listener
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

            println!("Listening on http://{}", listener.local_addr().unwrap());

            // Start the server
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

    Ok(())
}
