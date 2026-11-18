#![forbid(unsafe_code)]

use std::{env, error::Error};
use texcompiler::config::parse_bind_address;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let configured = match env::var("TEXCOMPILER_BIND") {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let address = parse_bind_address(configured.as_deref())?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(address = %listener.local_addr()?, "Rust service listening");
    let worker = env::var_os("TEXCOMPILER_WORKER")
        .map(std::path::PathBuf::from)
        .unwrap_or(env::current_exe()?.with_file_name("texcompiler-worker"));
    let app = texcompiler::native_router(worker).await;
    axum::serve(listener, app).await?;
    Ok(())
}
