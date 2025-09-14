pub mod routes;

use axum::{
    Router,
    http::Request,
    middleware::{self, Next},
    response::Response,
};

pub fn app_router() -> Router {
    Router::new()
        .nest("/api", routes::api_router())
        .layer(middleware::from_fn(error_handling_middleware))
}

/// Global error handling middleware to catch any unhandled errors
async fn error_handling_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, Response> {
    let uri = request.uri().clone();
    let method = request.method().clone();

    // Add request context to Sentry
    sentry::configure_scope(|scope| {
        scope.set_tag("endpoint", uri.path());
        scope.set_tag("method", method.as_str());
        scope.set_context(
            "request",
            sentry::protocol::Context::Other({
                let mut map = std::collections::BTreeMap::new();
                map.insert("path".to_string(), uri.path().into());
                map.insert("method".to_string(), method.to_string().into());
                if let Some(query) = uri.query() {
                    map.insert("query".to_string(), query.into());
                }
                map
            }),
        );
    });

    // Execute the request
    let response = next.run(request).await;

    // Check if response indicates an error that wasn't properly handled
    if response.status().is_server_error() {
        sentry::capture_message(
            &format!("Unhandled server error: {} {}", method, uri),
            sentry::Level::Error,
        );
    }

    Ok(response)
}
