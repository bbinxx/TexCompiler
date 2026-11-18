use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn readiness_fails_closed_when_worker_probe_fails() {
    let app = texcompiler::native_router("/missing-worker".into()).await;
    let response = app
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn unsupported_backend_is_rejected_instead_of_silently_running_native() {
    let app = texcompiler::native_router("/missing-worker".into()).await;
    let response = app
        .oneshot(
            Request::post("/api/v1/compile")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"code":"source","backend":"wasm"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
