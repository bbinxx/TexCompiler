use super::routes::ApiError;
use crate::{
    compiler::{Engine, detect_engine},
    workspace::{MAX_FILES, MAX_INPUT_BYTES, Project, validate_path},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

#[derive(Deserialize)]
pub(super) struct CompileRequest {
    pub code: Option<String>,
    pub files: Option<BTreeMap<String, String>>,
    #[serde(default = "default_main")]
    pub main_file: String,
    pub compiler: Option<String>,
    pub backend: Option<String>,
}
fn default_main() -> String {
    "document.tex".into()
}

pub(super) fn choose_engine(name: Option<&str>, code: &str) -> Result<Engine, ApiError> {
    match name.filter(|name| !name.is_empty()) {
        Some(name) => name.parse().map_err(|_| {
            ApiError::bad("Unsupported compiler", "Use pdflatex, xelatex or lualatex")
        }),
        None => Ok(detect_engine(code, Engine::PdfLatex)),
    }
}

pub(super) fn prepare(request: CompileRequest) -> Result<(Project, Engine, String), ApiError> {
    if request
        .backend
        .as_deref()
        .is_some_and(|backend| !matches!(backend, "native" | "auto"))
    {
        return Err(ApiError::bad(
            "Unsupported backend",
            "Only native and auto are available; WASM is not implemented",
        ));
    }
    let mut files = BTreeMap::new();
    let mut source = String::new();
    let main;
    if let Some(input) = request.files.filter(|f| !f.is_empty()) {
        main = request.main_file;
        for (name, content) in input {
            let extension = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            let binary = matches!(
                extension.as_str(),
                "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "webp"
                    | "pdf"
                    | "zip"
                    | "ttf"
                    | "otf"
                    | "woff"
                    | "woff2"
            );
            // Accept legacy base64 class/package assets and ordinary TeX text.
            let encoded_class =
                matches!(extension.as_str(), "cls" | "sty") && !content.contains(['\\', '%']);
            let bytes = if binary || encoded_class {
                STANDARD
                    .decode(content.trim())
                    .map_err(|_| ApiError::bad("Invalid project", "Invalid base64 asset"))?
            } else {
                content.into_bytes()
            };
            if matches!(extension.as_str(), "tex" | "cls" | "sty") {
                source.push_str(&String::from_utf8_lossy(&bytes));
                source.push('\n');
            }
            files.insert(name, bytes);
        }
    } else {
        let code = request
            .code
            .filter(|s| !s.is_empty())
            .ok_or_else(ApiError::empty)?;
        source = code.clone();
        main = "document.tex".to_owned();
        files.insert(main.clone(), code.into_bytes());
    }
    let engine = choose_engine(request.compiler.as_deref(), &source)?;
    let project = Project::new(files, main.clone())
        .map_err(|error| ApiError::bad("Invalid project", error.to_string()))?;
    Ok((project, engine, main))
}

pub(super) fn unzip(
    bytes: Vec<u8>,
    main: String,
    compiler: Option<String>,
) -> Result<(Project, Engine, String), ApiError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| ApiError::bad("Failed to unpack ZIP", "Invalid ZIP archive"))?;
    if archive.has_duplicate_names() {
        return Err(ApiError::bad("Invalid project", "Duplicate ZIP path"));
    }
    if archive.len() > MAX_FILES {
        return Err(ApiError::bad("Invalid project", "Too many archive entries"));
    }
    let mut files = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut total = 0usize;
    let mut source = String::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| ApiError::bad("Failed to unpack ZIP", "Invalid ZIP entry"))?;
        let directory = entry.is_dir();
        let name = entry
            .name()
            .map_err(|_| ApiError::bad("Invalid project", "Invalid ZIP filename"))?
            .trim_end_matches('/')
            .to_owned();
        validate_path(&name).map_err(|e| ApiError::bad("Invalid project", e.to_string()))?;
        if !seen.insert(name.clone()) {
            return Err(ApiError::bad("Invalid project", "Duplicate ZIP path"));
        }
        if let Some(mode) = entry.unix_mode()
            && !matches!(mode & 0o170000, 0 | 0o100000 | 0o040000)
        {
            return Err(ApiError::bad(
                "Invalid project",
                "ZIP links and special files are not supported",
            ));
        }
        if directory {
            continue;
        }
        let remaining = MAX_INPUT_BYTES - total;
        if entry.size() > remaining as u64 {
            return Err(ApiError::bad(
                "Invalid project",
                "Archive exceeds input limit",
            ));
        }
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take((remaining + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| ApiError::bad("Failed to unpack ZIP", "Corrupt ZIP entry"))?;
        if bytes.len() > remaining {
            return Err(ApiError::bad(
                "Invalid project",
                "Archive exceeds input limit",
            ));
        }
        total += bytes.len();
        if name.ends_with(".tex") {
            source.push_str(&String::from_utf8_lossy(&bytes));
            source.push('\n');
        }
        files.insert(name, bytes);
    }
    let engine = choose_engine(compiler.as_deref(), &source)?;
    let project = Project::new(files, main.clone())
        .map_err(|e| ApiError::bad("Invalid project", e.to_string()))?;
    Ok((project, engine, main))
}
