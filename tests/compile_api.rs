use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::path::PathBuf;
use tower::ServiceExt;

async fn unavailable_app() -> axum::Router {
    texcompiler::native_router(PathBuf::from("/texcompiler-missing-worker")).await
}
async fn request(app: axum::Router, path: &str, payload: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn compile_rejects_empty_input_with_legacy_error() {
    let (status, body) = request(unavailable_app().await, "/compile", json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, json!({"error":"No code provided"}));
}

#[tokio::test]
async fn compile_rejects_unsafe_projects_before_execution() {
    let (status, body) = request(
        unavailable_app().await,
        "/compile",
        json!({"files":{"../escape.tex":"source"},"main_file":"../escape.tex"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Invalid project");
}

#[tokio::test]
async fn unsupported_compiler_never_reaches_process_execution() {
    let (status, body) = request(
        unavailable_app().await,
        "/compile",
        json!({"code":"source","compiler":"/bin/sh"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Unsupported compiler");
}

#[tokio::test]
async fn unavailable_worker_returns_json_instead_of_internal_error() {
    let (status, body) = request(
        unavailable_app().await,
        "/compile",
        json!({"code":"source"}),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "Compilation service unavailable");
}

#[tokio::test]
async fn detection_supports_unicode_packages_and_ignores_comments() {
    for (code, want) in [
        (r"\usepackage{geometry,fontspec}", "lualatex"),
        ("% \\usepackage{fontspec}\nplain", "pdflatex"),
        (r"\usepackage{xeCJK}", "xelatex"),
    ] {
        let (status, body) = request(
            unavailable_app().await,
            "/detect-compiler",
            json!({"code":code}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"compiler":want}));
    }
}

#[tokio::test]
async fn ide_is_served_without_python_or_template_placeholders() {
    let app = unavailable_app().await;
    for path in ["/", "/static/js/main.js", "/static/css/style.css"] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("{{ version }}"));
    }
}

#[tokio::test]
#[ignore = "requires Linux sandbox and installed native TeX"]
async fn rust_http_returns_real_pdf_and_metadata() {
    let app =
        texcompiler::native_router(PathBuf::from(env!("CARGO_BIN_EXE_texcompiler-worker"))).await;
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let response=app.clone().oneshot(Request::post("/compile").header("content-type","application/json").body(Body::from(json!({"code":include_str!("../fixtures/basic/main.tex"),"compiler":engine}).to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/pdf");
        assert_eq!(response.headers()["x-texcompiler-engine"], engine);
        let bytes = to_bytes(response.into_body(), 32 * 1024 * 1024)
            .await
            .unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.get_pages().len(), 1);
        assert!(
            pdf.extract_text(&[1])
                .unwrap()
                .contains("TexCompiler compatibility baseline.")
        );
    }
}

fn zip_bytes(files: &[(&str, &str)]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, content) in files {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
fn bundle_request(bytes: &[u8], main: &str) -> Request<Body> {
    let mut body=format!("--texcompiler\r\nContent-Disposition: form-data; name=\"main_file\"\r\n\r\n{main}\r\n--texcompiler\r\nContent-Disposition: form-data; name=\"file\"; filename=\"project.zip\"\r\nContent-Type: application/zip\r\n\r\n").into_bytes();
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n--texcompiler--\r\n");
    Request::post("/compile/bundle")
        .header("content-type", "multipart/form-data; boundary=texcompiler")
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn zip_traversal_is_rejected_before_worker_availability() {
    let app = unavailable_app().await;
    let response = app
        .oneshot(bundle_request(
            &zip_bytes(&[("../escape.tex", "source")]),
            "main.tex",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap()["error"],
        "Invalid project"
    );
}

#[tokio::test]
#[ignore = "requires Linux sandbox and native TeX"]
async fn json_projects_and_zip_bundles_return_pdf() {
    let app =
        texcompiler::native_router(PathBuf::from(env!("CARGO_BIN_EXE_texcompiler-worker"))).await;
    let main = include_str!("../fixtures/multi-file/main.tex");
    let body = include_str!("../fixtures/multi-file/body.tex");
    let json_request = Request::post("/api/v1/projects/compile")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"files":{"main.tex":main,"body.tex":body},"main_file":"main.tex"}).to_string(),
        ))
        .unwrap();
    for request in [
        json_request,
        bundle_request(
            &zip_bytes(&[("main.tex", main), ("body.tex", body)]),
            "main.tex",
        ),
    ] {
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 32 * 1024 * 1024)
            .await
            .unwrap();
        assert!(
            lopdf::Document::load_mem(&bytes)
                .unwrap()
                .extract_text(&[1])
                .unwrap()
                .contains("Multi-file compatibility baseline.")
        );
    }
}

#[tokio::test]
#[ignore = "requires Linux sandbox and pdfLaTeX"]
async fn compile_api_honors_explicit_compiler_and_returns_diagnostics() {
    let app =
        texcompiler::native_router(PathBuf::from(env!("CARGO_BIN_EXE_texcompiler-worker"))).await;
    let (status,body)=request(app,"/compile",json!({"code":r"\documentclass{article}\usepackage{fontspec}\begin{document}Hello\end{document}","compiler":"pdflatex"})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Compilation failed");
    assert!(body["log"].as_str().unwrap().contains("fontspec"));
}

#[tokio::test]
async fn empty_browser_compiler_selection_uses_auto_detection() {
    let (status, body) = request(
        unavailable_app().await,
        "/compile",
        json!({"code":"source","compiler":""}),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "Compilation service unavailable");
}

#[tokio::test]
async fn duplicate_zip_names_are_rejected_before_library_deduplication() {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let archive=STANDARD.decode("UEsDBBQAAAAAAGCWcl1X7nGSBQAAAAUAAAAIAAAAbWFpbi50ZXhmaXJzdFBLAwQUAAAAAABglnJdaREftgYAAAAGAAAACAAAAG1haW4udGV4c2Vjb25kUEsBAhQDFAAAAAAAYJZyXVfucZIFAAAABQAAAAgAAAAAAAAAAAAAAIABAAAAAG1haW4udGV4UEsBAhQDFAAAAAAAYJZyXWkRH7YGAAAABgAAAAgAAAAAAAAAAAAAAIABKwAAAG1haW4udGV4UEsFBgAAAAACAAIAbAAAAFcAAAAAAA==").unwrap();
    let response = unavailable_app()
        .await
        .oneshot(bundle_request(&archive, "main.tex"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap()["error"],
        "Invalid project"
    );
}
