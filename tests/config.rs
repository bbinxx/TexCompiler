use std::net::SocketAddr;
use texcompiler::config::parse_bind_address;

#[test]
fn default_bind_is_local_and_separate_from_python() {
    assert_eq!(
        parse_bind_address(None).unwrap(),
        "127.0.0.1:8001".parse::<SocketAddr>().unwrap()
    );
}

#[test]
fn explicit_ipv4_and_ipv6_addresses_are_supported() {
    for value in ["0.0.0.0:9000", "[::1]:8001"] {
        assert_eq!(
            parse_bind_address(Some(value)).unwrap(),
            value.parse::<SocketAddr>().unwrap()
        );
    }
}

#[test]
fn invalid_bind_configuration_is_rejected() {
    for value in ["", "localhost:8001", "127.0.0.1:65536", "bad"] {
        assert!(parse_bind_address(Some(value)).is_err(), "{value}");
    }
}
