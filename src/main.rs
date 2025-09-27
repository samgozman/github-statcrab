mod cards;
mod github;
mod web;

use anyhow::Result;
use axum::{body::Body, http::Request};
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use sentry::integrations::tracing::EventFilter;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenvy::dotenv().ok();

    // Macros like #[tokio::main] are not supported. The Sentry client must be initialized before the async runtime is started.
    let guard = sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            // Performance monitoring configuration
            traces_sample_rate: 0.25,
            // Capture user IPs and potentially sensitive headers
            send_default_pii: true,
            // Custom tags for better organization
            default_integrations: true,
            // Set environment from ENV var or default to development
            environment: Some(
                std::env::var("SENTRY_ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .into(),
            ),
            // Enable automatic panic capture
            auto_session_tracking: true,
            ..Default::default()
        },
    ));

    let sentry_layer =
        sentry::integrations::tracing::layer().event_filter(|md| match *md.level() {
            // Capture error and warn level events as both logs and events in Sentry
            tracing::Level::ERROR | tracing::Level::WARN => EventFilter::Event | EventFilter::Log,
            // Ignore trace level events, as they're too verbose
            tracing::Level::TRACE => EventFilter::Ignore,
            // Capture everything else just as a log
            _ => EventFilter::Log,
        });
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_layer)
        .init();

    // Build our application with some routes
    let app = web::app_router().layer(
        ServiceBuilder::new()
            // Add Sentry tower layer for performance tracing
            .layer(sentry::integrations::tower::SentryLayer::new_from_top())
            // Bind a new Hub per request for error correlation
            .layer(NewSentryLayer::<Request<Body>>::new_from_top())
            // Start a transaction (Sentry root span) for each request
            .layer(SentryHttpLayer::new().enable_transaction()),
    );

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            // Get port from environment variable or default to 3000
            let port = std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<u16>()
                .expect("PORT must be a valid port number");

            // Bind address (0.0.0.0 to be accessible in containers; localhost otherwise)
            let addr: SocketAddr = ([0, 0, 0, 0], port).into();

            // Create the TCP listener
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    tracing::error!("Failed to bind to address {}: {}", addr, e);
                    sentry::capture_message(
                        &format!("Failed to bind to port 3000: {}", e),
                        sentry::Level::Fatal,
                    );
                    std::process::exit(1);
                }
            };

            tracing::info!("Listening on http://{}", listener.local_addr().unwrap());

            // Start the server with graceful shutdown
            if let Err(e) = axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(shutdown_signal())
                .await
            {
                tracing::error!("Server error: {}", e);
                sentry::capture_error(&e);
                std::process::exit(1);
            }

            tracing::info!("Server shutdown complete");
        });

    // Explicitly close Sentry client to ensure clean shutdown
    guard.close(Some(Duration::from_secs(2)));

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Received shutdown signal, shutting down gracefully...");
}
