use super::Engine;
use serde::{Deserialize, Serialize};

pub(crate) const MAX_REQUEST: usize = 8192;
pub(crate) const MAX_RESULT: usize = 32 * 1024 * 1024;
pub(crate) const MAX_DIAGNOSTICS: usize = 256 * 1024;
pub(crate) const MAX_PDF: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub engine: Engine,
    pub main_file: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Response {
    Success {
        pdf_base64: String,
        page_count: u32,
        engine: Engine,
        engine_version: String,
        method: String,
        log: String,
        warnings: Vec<String>,
    },
    Failure {
        log: String,
    },
}
