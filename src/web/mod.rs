pub mod routes;

use axum::Router;

pub fn app_router() -> Router {
    Router::new().nest("/api", routes::api_router())
}
