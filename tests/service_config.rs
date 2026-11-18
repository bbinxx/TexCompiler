use texcompiler::config::ServiceConfig;

#[test]
fn platform_port_and_explicit_bind_are_supported() {
    let platform = ServiceConfig::parse(None, Some("10000"), Some("a".repeat(32))).unwrap();
    assert_eq!(platform.bind.to_string(), "0.0.0.0:10000");
    let explicit = ServiceConfig::parse(Some("127.0.0.1:9000"), Some("bad"), None).unwrap();
    assert_eq!(explicit.bind.to_string(), "127.0.0.1:9000");
    assert_eq!(
        ServiceConfig::parse(None, None, None)
            .unwrap()
            .bind
            .to_string(),
        "127.0.0.1:8001"
    );
}

#[test]
fn remote_access_requires_a_nonempty_strong_token() {
    for token in [
        None,
        Some(String::new()),
        Some("short".into()),
        Some("a ".repeat(32)),
    ] {
        assert!(ServiceConfig::parse(Some("0.0.0.0:8000"), None, token).is_err());
    }
    assert!(ServiceConfig::parse(Some("[::]:8000"), None, Some("a".repeat(32))).is_ok());
    for port in ["0", "65536", "bad", ""] {
        assert!(ServiceConfig::parse(None, Some(port), Some("a".repeat(32))).is_err());
    }
}
