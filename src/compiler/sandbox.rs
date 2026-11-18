use std::{path::Path, process::Stdio};
use tokio::process::Command;

pub(crate) fn command(project: &Path, worker: &Path) -> Command {
    let mut command = Command::new("/usr/bin/bwrap");
    command.args([
        "--unshare-all",
        "--unshare-user",
        "--disable-userns",
        "--die-with-parent",
        "--new-session",
        "--cap-drop",
        "ALL",
        "--ro-bind",
        "/usr",
        "/usr",
        "--dev",
        "/dev",
    ]);
    // Match the host's system-library layout (Arch and Debian use different
    // /lib64 symlink targets). No project-controlled mount paths are accepted.
    for path in ["/bin", "/lib", "/lib64"] {
        let host = Path::new(path);
        if let Ok(target) = std::fs::read_link(host) {
            command.arg("--symlink").arg(target).arg(path);
        } else if host.is_dir() {
            command.args(["--ro-bind", path, path]);
        }
    }
    for path in [
        "/etc/fonts",
        "/etc/texmf",
        "/var/lib/texmf",
        "/etc/paperspecs",
        "/etc/papersize",
    ] {
        if Path::new(path).exists() {
            command.args(["--ro-bind", path, path]);
        }
    }
    command
        .arg("--ro-bind")
        .arg(project)
        .arg("/project")
        .arg("--ro-bind")
        .arg(worker)
        .arg("/worker")
        .args([
            "--size",
            "67108864",
            "--tmpfs",
            "/output",
            "--size",
            "33554432",
            "--tmpfs",
            "/tmp",
            "--remount-ro",
            "/",
            "--chdir",
            "/project",
            "--clearenv",
        ]);
    for (name, value) in [
        ("PATH", "/usr/bin"),
        ("HOME", "/tmp"),
        ("LANG", "C.UTF-8"),
        ("TEXMFVAR", "/tmp/texmf-var"),
        ("TEXMFCONFIG", "/tmp/texmf-config"),
        ("XDG_CACHE_HOME", "/tmp/cache"),
        ("openin_any", "p"),
        ("openout_any", "p"),
        ("shell_escape", "f"),
        ("TEXMFOUTPUT", "/output"),
        ("TEXCOMPILER_SANDBOX", "1"),
    ] {
        command.args(["--setenv", name, value]);
    }
    command.args([
        "--",
        "/usr/bin/prlimit",
        "--as=2147483648",
        "--cpu=120",
        "--fsize=16777216",
        "--nofile=128",
        "--core=0",
        "/worker",
    ]);
    command
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn helper(script: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("probe");
        fs::write(&path, format!("#!/bin/sh\nset -eu\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        (dir, path)
    }

    #[tokio::test]
    #[ignore = "requires Linux user namespaces and Bubblewrap/prlimit"]
    async fn isolates_files_environment_network_and_writes() {
        let host = tempfile::tempdir().unwrap();
        let marker = host.path().join("host-marker");
        fs::write(&marker, "private sentinel").unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let script = format!(
            r#"
[ ! -e '{}' ]
[ -z "${{TEXCOMPILER_TEST_SECRET+x}}" ]
[ ! -e /proc/self ]
! touch /project/write 2>/dev/null
! touch /usr/write 2>/dev/null
/usr/bin/python3 -c 'import socket; s=socket.socket(); s.settimeout(1); assert s.connect_ex(("127.0.0.1",{})) != 0'
printf 'isolated'
"#,
            marker.display(),
            port
        );
        let (_helper_dir, helper) = helper(&script);
        let project = tempfile::tempdir().unwrap();
        let output = command(project.path(), &helper)
            .env("TEXCOMPILER_TEST_SECRET", "sentinel")
            .output()
            .await
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"isolated");
    }

    #[tokio::test]
    #[ignore = "requires sized tmpfs support in Bubblewrap"]
    async fn output_tmpfs_has_an_enforced_aggregate_size() {
        let (_dir, helper) = helper(
            r#"
for n in 1 2 3 4 5 6 7 8; do
  dd if=/dev/zero of=/output/file$n bs=1048576 count=8 2>/dev/null
done
if dd if=/dev/zero of=/output/overflow bs=1048576 count=1 2>/dev/null; then exit 1; fi
printf 'bounded'
"#,
        );
        let project = tempfile::tempdir().unwrap();
        let output = command(project.path(), &helper).output().await.unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"bounded");
    }

    #[tokio::test]
    #[ignore = "requires Bubblewrap; helper deliberately missing inside namespace"]
    async fn failed_isolation_setup_never_executes_a_host_fallback() {
        let project = tempfile::tempdir().unwrap();
        let output = command(project.path(), Path::new("/texcompiler-missing-helper"))
            .output()
            .await
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}
