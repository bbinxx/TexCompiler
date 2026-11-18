use super::protocol::{self, Request, Response};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    process::{Command, Stdio},
};

/// Internal executable entry point. Only the trusted supervisor may invoke it
/// with project mounts and resource limits; this is not an HTTP handler.
///
/// # Errors
/// Returns IO/protocol errors; normal compilation failures are encoded on stdout.
pub fn worker_main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("TEXCOMPILER_SANDBOX").as_deref() != Ok("1")
        || std::env::current_dir()? != Path::new("/project")
    {
        return Err("worker requires its private sandbox".into());
    }
    let mut bytes = Vec::new();
    io::stdin()
        .take((protocol::MAX_REQUEST + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > protocol::MAX_REQUEST {
        return Err("request exceeds limit".into());
    }
    let request: Request = serde_json::from_slice(&bytes)?;
    crate::workspace::validate_path(&request.main_file)?;
    if !request.main_file.ends_with(".tex") {
        return Err("main file must be .tex".into());
    }
    let result = compile(&request).unwrap_or_else(|_| Response::Failure {
        log: "native worker could not prepare or validate output".into(),
    });
    serde_json::to_writer(io::stdout().lock(), &result)?;
    Ok(())
}

fn read_bounded(path: &Path, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)?;
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    Ok((bytes, truncated))
}

fn diagnostics() -> String {
    let mut log = String::new();
    for path in ["/output/document.log", "/output/process.log"] {
        let (bytes, truncated) =
            read_bounded(Path::new(path), protocol::MAX_DIAGNOSTICS / 2 - 128).unwrap_or_default();
        log.push_str(&String::from_utf8_lossy(&bytes));
        if truncated {
            log.push_str("\n[diagnostics truncated]\n");
        }
        log.push('\n');
    }
    log.truncate(log.floor_char_boundary(protocol::MAX_DIAGNOSTICS));
    log
}

fn compile(request: &Request) -> Result<Response, Box<dyn std::error::Error>> {
    let executable = request.engine.executable();
    if !Path::new(executable).is_file() {
        return Ok(Response::Failure {
            log: "requested engine is not installed".into(),
        });
    }
    let version_file = File::create("/output/version.log")?;
    let status = Command::new(executable)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(version_file.try_clone()?)
        .stderr(version_file)
        .status()?;
    if !status.success() {
        return Ok(Response::Failure {
            log: "engine version probe failed".into(),
        });
    }
    let (version, _) = read_bounded(Path::new("/output/version.log"), 16 * 1024)?;
    let engine_version = String::from_utf8_lossy(&version)
        .lines()
        .next()
        .unwrap_or("")
        .to_owned();
    let latexmk = Path::new("/usr/bin/latexmk").is_file();
    for _ in 0..if latexmk { 1 } else { 3 } {
        let output = File::create("/output/process.log")?;
        let status = Command::new(if latexmk {
            "/usr/bin/latexmk"
        } else {
            executable
        })
        .args(request.engine.arguments(&request.main_file, latexmk))
        .current_dir("/project")
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(output)
        .status()?;
        if !status.success() {
            return Ok(Response::Failure { log: diagnostics() });
        }
    }
    let (pdf, truncated) = read_bounded(Path::new("/output/document.pdf"), protocol::MAX_PDF)?;
    if truncated {
        return Ok(Response::Failure {
            log: "PDF exceeds output limit".into(),
        });
    }
    let document = lopdf::Document::load_mem(&pdf)?;
    let page_count = u32::try_from(document.get_pages().len())?;
    if page_count == 0 {
        return Ok(Response::Failure {
            log: "output PDF has no pages".into(),
        });
    }
    let log = diagnostics();
    let warnings = log
        .lines()
        .filter(|line| line.contains("Warning:"))
        .map(str::to_string)
        .collect();
    // Drop parsed objects before allocating the base64 transport envelope.
    drop(document);
    Ok(Response::Success {
        pdf_base64: STANDARD.encode(pdf),
        page_count,
        engine: request.engine,
        engine_version,
        method: if latexmk {
            "latexmk"
        } else {
            "direct-three-pass"
        }
        .into(),
        log,
        warnings,
    })
}
