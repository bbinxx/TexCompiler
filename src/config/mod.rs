//! Bind configuration for a separate development listener.
use std::net::{AddrParseError, SocketAddr};

/// Parse TEXCOMPILER_BIND, defaulting to a loopback-only development port.
///
/// # Errors
/// Returns an error if the value is not a numeric IP address and port.
pub fn parse_bind_address(value: Option<&str>) -> Result<SocketAddr, AddrParseError> {
    value.unwrap_or("127.0.0.1:8001").parse()
}
