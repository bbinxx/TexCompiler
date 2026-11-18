//! Experimental native compilation; not wired to HTTP compilation routes.
mod engine;
pub use engine::Engine;
mod native;
pub use native::{Compilation, NativeCompiler, NativeError};
mod protocol;
mod sandbox;
mod worker;
pub use worker::worker_main;
mod detection;
pub use detection::detect_engine;
