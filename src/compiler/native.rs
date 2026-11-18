use super::{
    Engine,
    protocol::{self, Request, Response},
    sandbox,
};
use crate::workspace::Project;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{io, path::PathBuf, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Child,
    sync::{Semaphore, oneshot},
};

#[derive(Debug)]
pub struct Compilation {
    pub pdf: Vec<u8>,
    pub page_count: u32,
    pub engine: Engine,
    pub engine_version: String,
    pub method: String,
    pub log: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum NativeError {
    #[error("native worker or isolation tools unavailable")]
    Unavailable,
    #[error("compilation timed out")]
    Timeout,
    #[error("all native worker slots are busy")]
    Busy,
    #[error("compilation failed: {0}")]
    Compilation(String),
    #[error("sandbox process failed: {0}")]
    Sandbox(String),
    #[error("invalid or oversized worker response")]
    Protocol,
    #[error("native worker IO failed")]
    Io(#[from] io::Error),
}

#[derive(Clone)]
pub struct NativeCompiler {
    worker: PathBuf,
    timeout: Duration,
    slots: Arc<Semaphore>,
}
impl NativeCompiler {
    /// Configure a trusted same-build helper, never a path supplied by a client.
    pub fn new(worker: PathBuf) -> Self {
        Self {
            worker,
            timeout: Duration::from_secs(120),
            slots: Arc::new(Semaphore::new(2)),
        }
    }

    /// Shorten the wall deadline. Values above the 120-second ceiling are capped.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.min(Duration::from_secs(120));
        self
    }

    /// Compile in a private Linux sandbox. Cancelling the caller triggers cleanup
    /// in the supervising task; its permit/workspace remain owned until reaping.
    ///
    /// # Errors
    /// Fails closed on missing isolation tools, invalid output, busy workers,
    /// timeouts, process errors or unsuccessful TeX compilation.
    pub async fn compile(
        &self,
        project: Project,
        engine: Engine,
    ) -> Result<Compilation, NativeError> {
        let permit = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| NativeError::Busy)?;
        let worker = self
            .worker
            .canonicalize()
            .map_err(|_| NativeError::Unavailable)?;
        if !worker.is_file()
            || !cfg!(target_os = "linux")
            || !["/usr/bin/bwrap", "/usr/bin/prlimit"]
                .iter()
                .all(|p| std::path::Path::new(p).is_file())
        {
            return Err(NativeError::Unavailable);
        }
        let request = serde_json::to_vec(&Request {
            engine,
            main_file: project.main_file().into(),
        })
        .map_err(|_| NativeError::Protocol)?;
        if request.len() > protocol::MAX_REQUEST {
            return Err(NativeError::Protocol);
        }
        let deadline = tokio::time::Instant::now() + self.timeout;
        let (mut sender, receiver) = oneshot::channel();
        tokio::spawn(async move {
            // The owner task retains the permit even if the caller cancels
            // while private workspace creation is still in spawn_blocking.
            let result = async {
                if sender.is_closed() {
                    return Err(NativeError::Protocol);
                }
                let workspace = tokio::task::spawn_blocking(move || project.prepare())
                    .await
                    .map_err(|_| NativeError::Protocol)??;
                if sender.is_closed() {
                    return Err(NativeError::Protocol);
                }
                if tokio::time::Instant::now() >= deadline {
                    return Err(NativeError::Timeout);
                }
                let mut child = sandbox::command(workspace.path(), &worker).spawn()?;
                let result = tokio::select! {
                    result = exchange(&mut child, &request) => result,
                    _ = tokio::time::sleep_until(deadline) => Err(NativeError::Timeout),
                    _ = sender.closed() => Err(NativeError::Protocol),
                };
                if child.try_wait().ok().flatten().is_none() {
                    let _ = child.kill().await;
                }
                let _ = child.wait().await;
                drop(workspace);
                result.and_then(|bytes| decode(&bytes, engine))
            }
            .await;
            drop(permit);
            let _ = sender.send(result);
        });
        receiver.await.map_err(|_| NativeError::Protocol)?
    }
}

async fn bounded_read(
    mut reader: impl AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, NativeError> {
    let mut bytes = Vec::new();
    (&mut reader)
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > limit {
        return Err(NativeError::Protocol);
    }
    Ok(bytes)
}

async fn exchange(child: &mut Child, request: &[u8]) -> Result<Vec<u8>, NativeError> {
    let mut stdin = child.stdin.take().ok_or(NativeError::Protocol)?;
    let stdout = child.stdout.take().ok_or(NativeError::Protocol)?;
    let stderr = child.stderr.take().ok_or(NativeError::Protocol)?;
    let write = async {
        stdin.write_all(request).await?;
        stdin.shutdown().await?;
        drop(stdin);
        Ok::<_, NativeError>(())
    };
    let (_, out, err, status) = tokio::try_join!(
        write,
        bounded_read(stdout, protocol::MAX_RESULT),
        bounded_read(stderr, 64 * 1024),
        async { child.wait().await.map_err(NativeError::Io) }
    )?;
    if !status.success() {
        return Err(NativeError::Sandbox(
            String::from_utf8_lossy(&err).into_owned(),
        ));
    }
    Ok(out)
}

fn decode(bytes: &[u8], expected: Engine) -> Result<Compilation, NativeError> {
    match serde_json::from_slice::<Response>(bytes).map_err(|_| NativeError::Protocol)? {
        Response::Failure { log } => {
            if log.len() > protocol::MAX_DIAGNOSTICS {
                return Err(NativeError::Protocol);
            }
            Err(NativeError::Compilation(log))
        }
        Response::Success {
            pdf_base64,
            page_count,
            engine,
            engine_version,
            method,
            log,
            warnings,
        } => {
            let pdf = STANDARD
                .decode(pdf_base64)
                .map_err(|_| NativeError::Protocol)?;
            if pdf.len() > protocol::MAX_PDF
                || !pdf.starts_with(b"%PDF-")
                || page_count == 0
                || engine != expected
                || log.len() > protocol::MAX_DIAGNOSTICS
                || engine_version.len() > 16 * 1024
                || warnings.iter().map(String::len).sum::<usize>() > protocol::MAX_DIAGNOSTICS
                || !matches!(method.as_str(), "latexmk" | "direct-three-pass")
            {
                return Err(NativeError::Protocol);
            }
            Ok(Compilation {
                pdf,
                page_count,
                engine,
                engine_version,
                method,
                log,
                warnings,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_failure_diagnostics() {
        let bytes = serde_json::to_vec(&Response::Failure {
            log: "x".repeat(protocol::MAX_DIAGNOSTICS + 1),
        })
        .unwrap();
        assert!(matches!(
            decode(&bytes, Engine::PdfLatex),
            Err(NativeError::Protocol)
        ));
    }

    #[tokio::test]
    async fn protocol_streams_reject_overflow_without_waiting_for_eof() {
        let (mut writer, reader) = tokio::io::duplex(64);
        writer.write_all(b"four").await.unwrap();
        assert!(matches!(
            bounded_read(reader, 3).await,
            Err(NativeError::Protocol)
        ));
    }

    #[cfg(target_os = "linux")]
    mod lifecycle {
        use super::*;
        use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt, path::Path};

        fn project() -> Project {
            Project::new(
                BTreeMap::from([("main.tex".into(), b"source".to_vec())]),
                "main.tex".into(),
            )
            .unwrap()
        }
        fn helper() -> (tempfile::TempDir, PathBuf) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("helper");
            fs::write(
                &path,
                "#!/bin/sh\ncat >/dev/null\nsleep 30 &\nsleep 30 &\nwait\n",
            )
            .unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            (dir, path)
        }
        fn find_sandbox(helper: &Path) -> Option<(u32, PathBuf)> {
            for entry in fs::read_dir("/proc").unwrap().flatten() {
                let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                    continue;
                };
                let Ok(bytes) = fs::read(entry.path().join("cmdline")) else {
                    continue;
                };
                let args: Vec<_> = bytes
                    .split(|b| *b == 0)
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .collect();
                if args.first().is_some_and(|arg| arg == "/usr/bin/bwrap")
                    && args.iter().any(|a| Path::new(a) == helper)
                {
                    for triple in args.windows(3) {
                        if triple[0] == "--ro-bind" && triple[2] == "/project" {
                            return Some((pid, PathBuf::from(&triple[1])));
                        }
                    }
                }
            }
            None
        }
        fn descendants(pid: u32, all: &mut Vec<u32>) {
            if let Ok(children) = fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
                for child in children
                    .split_whitespace()
                    .filter_map(|s| s.parse::<u32>().ok())
                {
                    all.push(child);
                    descendants(child, all);
                }
            }
        }
        async fn started(helper: &Path) -> (Vec<u32>, PathBuf) {
            for _ in 0..200 {
                if let Some((pid, path)) = find_sandbox(helper) {
                    let mut all = vec![pid];
                    descendants(pid, &mut all);
                    if all.len() >= 4 {
                        return (all, path);
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            panic!("sandbox did not start descendants");
        }
        async fn cleaned(pids: &[u32], workspace: &Path, compiler: &NativeCompiler) {
            for _ in 0..200 {
                if pids
                    .iter()
                    .all(|pid| !Path::new(&format!("/proc/{pid}")).exists())
                    && !workspace.exists()
                    && compiler.slots.available_permits() == 2
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            panic!("sandbox descendants, workspace or slot survived cleanup");
        }

        #[tokio::test]
        #[ignore = "requires Linux Bubblewrap and externally observable /proc"]
        async fn cancellation_reaps_descendants_before_releasing_workspace_and_slot() {
            let (_dir, helper) = helper();
            let compiler = NativeCompiler::new(helper.clone());
            let job_compiler = compiler.clone();
            let job =
                tokio::spawn(
                    async move { job_compiler.compile(project(), Engine::PdfLatex).await },
                );
            let (pids, path) = started(&helper).await;
            assert!(path.exists());
            assert_eq!(compiler.slots.available_permits(), 1);
            job.abort();
            let _ = job.await;
            cleaned(&pids, &path, &compiler).await;
        }

        #[tokio::test]
        #[ignore = "requires Linux Bubblewrap and externally observable /proc"]
        async fn timeout_reaps_descendants_before_returning() {
            let (_dir, helper) = helper();
            let compiler = NativeCompiler::new(helper.clone()).with_timeout(Duration::from_secs(2));
            let job_compiler = compiler.clone();
            let job =
                tokio::spawn(
                    async move { job_compiler.compile(project(), Engine::PdfLatex).await },
                );
            let (pids, path) = started(&helper).await;
            assert!(matches!(job.await.unwrap(), Err(NativeError::Timeout)));
            cleaned(&pids, &path, &compiler).await;
        }

        #[tokio::test]
        #[ignore = "requires Linux Bubblewrap"]
        async fn worker_slots_fail_fast_when_busy() {
            let (_dir, helper) = helper();
            let compiler = NativeCompiler::new(helper.clone());
            let _one = compiler.slots.clone().acquire_owned().await.unwrap();
            let _two = compiler.slots.clone().acquire_owned().await.unwrap();
            assert!(matches!(
                compiler.compile(project(), Engine::PdfLatex).await,
                Err(NativeError::Busy)
            ));
        }
    }
}
