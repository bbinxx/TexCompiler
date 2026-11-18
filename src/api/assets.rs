use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};

pub(super) async fn index() -> Html<String> {
    Html(
        include_str!("../../web/templates/index.html")
            .replace("{{ version }}", env!("CARGO_PKG_VERSION")),
    )
}
pub(super) async fn asset(Path(name): Path<String>) -> Response {
    let files: &[(&str, &[u8])] = &[
        (
            "js/project.js",
            include_bytes!("../../web/static/js/project.js"),
        ),
        ("js/auth.js", include_bytes!("../../web/static/js/auth.js")),
        (
            "css/style.css",
            include_bytes!("../../web/static/css/style.css"),
        ),
        (
            "js/compiler.js",
            include_bytes!("../../web/static/js/compiler.js"),
        ),
        (
            "js/defaults.js",
            include_bytes!("../../web/static/js/defaults.js"),
        ),
        (
            "js/detector.js",
            include_bytes!("../../web/static/js/detector.js"),
        ),
        (
            "js/editor.js",
            include_bytes!("../../web/static/js/editor.js"),
        ),
        (
            "js/explorer.js",
            include_bytes!("../../web/static/js/explorer.js"),
        ),
        ("js/main.js", include_bytes!("../../web/static/js/main.js")),
        (
            "js/settings.js",
            include_bytes!("../../web/static/js/settings.js"),
        ),
        (
            "js/terminal.js",
            include_bytes!("../../web/static/js/terminal.js"),
        ),
        ("js/ui.js", include_bytes!("../../web/static/js/ui.js")),
    ];
    match files.iter().find(|(path, _)| *path == name) {
        Some((_, bytes)) => (
            [(
                header::CONTENT_TYPE,
                if name.ends_with(".css") {
                    "text/css; charset=utf-8"
                } else {
                    "text/javascript; charset=utf-8"
                },
            )],
            *bytes,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
