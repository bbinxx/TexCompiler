//! Parallel Rust API foundation. Compilation remains served by Python.
#![forbid(unsafe_code)]

mod api;
pub mod config;

/// Build the versioned foundation routes without enabling compilation.
pub fn router() -> axum::Router {
    api::router()
}
