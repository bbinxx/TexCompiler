//! Rust API foundation and experimental isolated Linux native worker.
//! Production HTTP compilation remains served by Python.
#![forbid(unsafe_code)]

mod api;
pub mod config;

/// Build the versioned foundation routes without enabling compilation.
pub fn router() -> axum::Router {
    api::router()
}

pub mod compiler;
pub mod workspace;

/// Build the usable experimental service, probing worker isolation on startup.
/// Use on loopback with trusted documents while production hardening continues.
pub async fn native_router(worker: std::path::PathBuf) -> axum::Router {
    api::native_router(worker).await
}
