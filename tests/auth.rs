use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn protected_routes_require_exact_bearer_token_and_public_health_stays_available() {
    let token = "a".repeat(32);
    let app = texcompiler::configured_router("/missing-worker".into(), Some(token.clone())).await;
    for header in [
        None,
        Some("Bearer wrong".to_owned()),
        Some(format!("Basic {token}")),
    ] {
        let mut req = Request::post("/compile").header("content-type", "application/json");
        if let Some(header) = header {
            req = req.header("authorization", header);
        }
        let response = app
            .clone()
            .oneshot(req.body(Body::from("{}")).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains(&token));
    }
    let response = app
        .clone()
        .oneshot(
            Request::post("/compile")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    for path in ["/", "/health", "/api/v1/health", "/compilers"] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
