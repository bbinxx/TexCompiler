use super::{
    assets,
    project::{self, CompileRequest},
};
use crate::{
    compiler::{Engine, NativeCompiler, NativeError, detect_engine},
    workspace::Project,
};
use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Multipart, State, multipart::MultipartRejection, rejection::JsonRejection,
    },
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const BODY_LIMIT: usize = 32 * 1024 * 1024;
#[derive(Clone)]
struct App {
    compiler: NativeCompiler,
    ready: bool,
    engines: Vec<&'static str>,
}

pub(crate) async fn router(worker: PathBuf) -> Router {
    let compiler = NativeCompiler::new(worker);
    let mut engines = [Engine::PdfLatex, Engine::XeLatex, Engine::LuaLatex]
        .into_iter()
        .filter(|engine| Path::new(engine.executable()).is_file())
        .map(name)
        .collect::<Vec<_>>();
    engines.sort();
    let probe = Project::new(
        std::collections::BTreeMap::from([(
            "main.tex".into(),
            b"\\documentclass{article}\\begin{document}Ready\\end{document}".to_vec(),
        )]),
        "main.tex".into(),
    );
    let preferred = if engines.contains(&"pdflatex") {
        Some(Engine::PdfLatex)
    } else {
        engines
            .first()
            .and_then(|engine| engine.parse::<Engine>().ok())
    };
    let ready = if let (Ok(project), Some(engine)) = (probe, preferred) {
        match compiler.compile(project, engine).await {
            Ok(_) => true,
            Err(error) => {
                tracing::warn!(%error,"native startup probe failed");
                false
            }
        }
    } else {
        false
    };
    let state = App {
        compiler,
        ready,
        engines,
    };
    Router::new()
        .route("/", get(assets::index))
        .route("/static/{*name}", get(assets::asset))
        .route("/compile", post(compile))
        .route("/compile/bundle", post(bundle))
        .route("/detect-compiler", post(detect))
        .route("/compilers", get(compilers))
        .route("/health", get(health))
        .route("/api/v1/compile", post(compile))
        .route("/api/v1/projects/compile", post(compile))
        .route("/api/v1/engines", get(compilers))
        .route("/api/v1/health", get(health))
        .route("/api/v1/capabilities", get(capabilities))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}
fn name(engine: Engine) -> &'static str {
    match engine {
        Engine::PdfLatex => "pdflatex",
        Engine::XeLatex => "xelatex",
        Engine::LuaLatex => "lualatex",
    }
}

pub(super) struct ApiError {
    status: StatusCode,
    body: Value,
}
impl ApiError {
    pub(super) fn bad(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: json!({"error":error.into(),"details":details.into()}),
        }
    }
    pub(super) fn empty() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: json!({"error":"No code provided"}),
        }
    }
    fn unavailable() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: json!({"error":"Compilation service unavailable","details":"Build both Rust binaries and check native dependencies and server logs."}),
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

async fn compile(
    State(app): State<App>,
    request: Result<Json<CompileRequest>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) = request.map_err(|e| ApiError {
        status: e.status(),
        body: json!({"error":"Invalid compile request"}),
    })?;
    let prepared = tokio::task::spawn_blocking(move || project::prepare(request))
        .await
        .map_err(|_| ApiError::unavailable())??;
    execute(app, prepared).await
}

async fn bundle(
    State(app): State<App>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Response, ApiError> {
    let mut multipart =
        multipart.map_err(|_| ApiError::bad("Invalid bundle", "Expected multipart form data"))?;
    let mut file = None;
    let mut main = "main.tex".to_owned();
    let mut engine = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad("Invalid bundle", "Malformed multipart data"))?
    {
        match field.name().unwrap_or("") {
            "file" => {
                if file.is_some() {
                    return Err(ApiError::bad("Invalid bundle", "Only one ZIP is supported"));
                }
                if !field.file_name().is_some_and(|name| name.ends_with(".zip")) {
                    return Err(ApiError {
                        status: StatusCode::BAD_REQUEST,
                        body: json!({"detail":"Only ZIP bundles are supported."}),
                    });
                }
                file = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| {
                            ApiError::bad(
                                "Invalid bundle",
                                "Upload exceeds limit or could not be read",
                            )
                        })?
                        .to_vec(),
                );
            }
            "main_file" => {
                main = field
                    .text()
                    .await
                    .map_err(|_| ApiError::bad("Invalid bundle", "Invalid main_file"))?
            }
            "compiler" => {
                engine = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| ApiError::bad("Invalid bundle", "Invalid compiler"))?,
                )
            }
            _ => {}
        }
    }
    let bytes = file.ok_or_else(|| ApiError::bad("Invalid bundle", "Missing ZIP file"))?;
    let prepared = tokio::task::spawn_blocking(move || project::unzip(bytes, main, engine))
        .await
        .map_err(|_| ApiError::unavailable())??;
    execute(app, prepared).await
}

async fn execute(
    app: App,
    (project, engine, main): (Project, Engine, String),
) -> Result<Response, ApiError> {
    if !app.ready {
        return Err(ApiError::unavailable());
    }
    let output=app.compiler.compile(project,engine).await.map_err(|error| match error {
        NativeError::Compilation(log)=>ApiError {status:StatusCode::BAD_REQUEST,body:json!({"error":"Compilation failed","log":log,"warnings":[],"details":"Review the log for missing packages or syntax errors."})},
        NativeError::Timeout=>ApiError {status:StatusCode::BAD_REQUEST,body:json!({"error":"Compilation failed","log":"Compilation timed out after 120 seconds.","warnings":[]})},
        NativeError::Busy=>ApiError {status:StatusCode::TOO_MANY_REQUESTS,body:json!({"error":"Compiler busy","details":"Try again after a current compilation finishes."})},
        error=>{tracing::warn!(%error,"native compilation service error");ApiError::unavailable()},
    })?;
    let stem = Path::new(&main)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let safe = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let mut response = output.pdf.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename={safe}.pdf"))
            .map_err(|_| ApiError::unavailable())?,
    );
    headers.insert(
        "x-texcompiler-engine",
        HeaderValue::from_static(name(output.engine)),
    );
    headers.insert(
        "x-texcompiler-pages",
        HeaderValue::from_str(&output.page_count.to_string())
            .map_err(|_| ApiError::unavailable())?,
    );
    headers.insert(
        "x-texcompiler-method",
        HeaderValue::from_str(&output.method).map_err(|_| ApiError::unavailable())?,
    );
    Ok(response)
}
#[derive(Deserialize)]
struct DetectRequest {
    code: String,
    preferred: Option<String>,
}
async fn detect(
    request: Result<Json<DetectRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let Json(request) = request.map_err(|_| {
        ApiError::bad(
            "Invalid detection request",
            "Expected code and optional preferred compiler",
        )
    })?;
    let preferred =
        project::choose_engine(Some(request.preferred.as_deref().unwrap_or("pdflatex")), "")?;
    Ok(Json(
        json!({"compiler":name(detect_engine(&request.code,preferred))}),
    ))
}
async fn compilers(State(app): State<App>) -> Json<Value> {
    let mut engines = app.engines;
    engines.sort();
    Json(json!({"compilers":engines}))
}
async fn health(State(app): State<App>) -> Json<Value> {
    Json(
        json!({"status":if app.ready {"healthy"}else{"degraded"},"version":env!("CARGO_PKG_VERSION"),"compilers":app.engines,"latexmk":if Path::new("/usr/bin/latexmk").is_file(){"/usr/bin/latexmk"}else{"Not found"},"compilation_ready":app.ready,"production_ready":false}),
    )
}
async fn capabilities(State(app): State<App>) -> Json<Value> {
    Json(
        json!({"backends":if app.ready {vec!["native"]}else{vec![]},"compilation_ready":app.ready,"production_ready":false,"experimental":true}),
    )
}
