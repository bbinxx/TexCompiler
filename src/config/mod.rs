//! Bind configuration for a separate development listener.
use std::net::{AddrParseError, SocketAddr};

/// Parse TEXCOMPILER_BIND, defaulting to a loopback-only development port.
///
/// # Errors
/// Returns an error if the value is not a numeric IP address and port.
pub fn parse_bind_address(value: Option<&str>) -> Result<SocketAddr, AddrParseError> {
    value.unwrap_or("127.0.0.1:8001").parse()
}

/// Startup settings. Tokens are deliberately not included in debug output.
pub struct ServiceConfig {
    pub bind: SocketAddr,
    pub token: Option<String>,
}

impl ServiceConfig {
    /// Explicit bind takes precedence over platform PORT. An unset or empty
    /// token allows unauthenticated compilation on any listener.
    ///
    /// # Errors
    /// Rejects invalid listeners, platform ports and invalid nonempty tokens.
    pub fn parse(
        bind: Option<&str>,
        port: Option<&str>,
        token: Option<String>,
    ) -> Result<Self, String> {
        let bind = if let Some(bind) = bind {
            parse_bind_address(Some(bind)).map_err(|_| "invalid TEXCOMPILER_BIND")?
        } else if let Some(port) = port {
            let port = port.parse::<u16>().map_err(|_| "invalid PORT")?;
            if port == 0 {
                return Err("PORT must be nonzero".into());
            }
            SocketAddr::from(([0, 0, 0, 0], port))
        } else {
            parse_bind_address(None).map_err(|_| "invalid default bind")?
        };
        let token = token.filter(|value| !value.is_empty());
        if token.as_ref().is_some_and(|value| {
            value.len() < 32
                || value.len() > 256
                || !value.bytes().all(|byte| byte.is_ascii_graphic())
        }) {
            return Err(
                "TEXCOMPILER_API_TOKEN must contain 32–256 non-whitespace ASCII characters".into(),
            );
        }
        Ok(Self { bind, token })
    }
}
