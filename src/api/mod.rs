use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    compilation_ready: bool,
}

#[derive(Serialize)]
struct Capabilities {
    backends: [&'static str; 0],
    compilation_ready: bool,
}

pub(crate) fn router() -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/capabilities", get(capabilities))
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "starting",
        compilation_ready: false,
    })
}

async fn capabilities() -> Json<Capabilities> {
    Json(Capabilities {
        backends: [],
        compilation_ready: false,
    })
}
mod assets;
mod project;
mod service;
pub(crate) use service::router as native_router;
