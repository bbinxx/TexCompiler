use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get(path: &str) -> (StatusCode, Value) {
    let response = texcompiler::router()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn health_does_not_claim_compilation_readiness() {
    let (status, value) = get("/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        value,
        json!({"status":"starting", "compilation_ready":false})
    );
}

#[tokio::test]
async fn capabilities_advertise_no_available_backends() {
    let (status, value) = get("/api/v1/capabilities").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value, json!({"backends":[], "compilation_ready":false}));
}

#[tokio::test]
async fn unknown_and_legacy_routes_are_not_implemented() {
    for path in ["/unknown", "/compile", "/health"] {
        assert_eq!(get(path).await.0, StatusCode::NOT_FOUND);
    }
}
