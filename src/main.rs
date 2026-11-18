#![forbid(unsafe_code)]

use std::{env, error::Error};
use texcompiler::config::ServiceConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let read = |name| match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error),
    };
    let config = ServiceConfig::parse(
        read("TEXCOMPILER_BIND")?.as_deref(),
        read("PORT")?.as_deref(),
        read("TEXCOMPILER_API_TOKEN")?,
    )?;
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %listener.local_addr()?, "Rust service listening");
    let worker = env::var_os("TEXCOMPILER_WORKER")
        .map(std::path::PathBuf::from)
        .unwrap_or(env::current_exe()?.with_file_name("texcompiler-worker"));
    let app = texcompiler::configured_router(worker, config.token).await;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            #[cfg(unix)]
            {
                let terminate =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());
                if let Ok(mut terminate) = terminate {
                    tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
                    return;
                }
            }
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
