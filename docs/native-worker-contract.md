# Experimental native worker contract

Status: experimental implementation on `feature/native-worker-core`; not production
readiness. HTTP compilation remains disabled until follow-up deployment gates.

## Boundary and protocol

The Rust supervisor accepts in-memory project files (relative path -> bytes),
a main `.tex` entry and an explicit enum: pdflatex, xelatex or lualatex. No
arbitrary executables, shell command, source rewriting or engine substitution.
Paths reject empty/dot/parent segments, absolute/drive/UNC paths, backslashes,
control characters and option-like components. Limit 256 files and 20 MiB total, paths to 2048 bytes and components to 255 bytes.
Validate the entire project, main-file existence and file/directory collisions
before creating a private tempfile workspace. Only regular files are created;
no externally writable project root or archive extraction is accepted here.

A same-build `texcompiler-worker` Rust helper runs inside Linux Bubblewrap. stdin is a bounded JSON
request (engine and main path); stdout is a bounded JSON result containing PDF
base64, logs, warnings, page count, engine version and runner method, or a typed
failure. Compiler stdout/stderr go to bounded sandbox files, not the protocol.
No HTTP route or capability advertises the experimental worker yet.

## Execution policy

- Require /usr/bin/bwrap and /usr/bin/prlimit; fail closed when unavailable or
  isolation setup fails. Never fall back to running TeX on the host.
- New user/mount/PID/network/IPC/UTS namespaces, no capabilities, new session,
  parent-death termination, and no nested user namespaces. Do not mount /proc.
- Mount /usr read-only, preserving host /bin,/lib,/lib64 symlink or read-only
  library directory layouts. Expose approved /etc/fonts, /etc/texmf and
  /var/lib/texmf and libpaper /etc/paperspecs,/etc/papersize read-only when present; no home, credentials or host /tmp.
- Bind only the private input workspace read-only at /project and the trusted
  worker executable read-only at /worker. Mount /output (64 MiB) and /tmp
  (32 MiB) as sized tmpfs; restrict root writes by remounting it read-only.
- Clear inherited environment; fixed PATH, HOME=/tmp, font/TeX caches under
  /tmp, paranoid TeX read/write policies and shell escape disabled.
- Inside the sandbox prlimit enforces 2 GiB address-space per process, 120 CPU
  seconds per process, 16 MiB per file, 128 descriptors and no core dumps.
- Supervisor deadline 120 seconds (cleanup is awaited even after expiration),
  8 KiB stdin / 32 MiB JSON result / 64 KiB stderr protocol bounds, maximum
  PDF 16 MiB and diagnostics 256 KiB. Two jobs per supervisor, fail fast when
  busy. Timeout and cancellation kill and reap the child/PID namespace. A detached
  owner retains the workspace and slot through preparation and cleanup even
  when the caller drops its future.
- Use latexmk with -norc, -no-shell-escape and engine-specific flags when
  installed. Otherwise three direct passes with -no-shell-escape,
  -halt-on-error, fixed jobname=document and private output directory. Direct
  fallback does not promise bibliography/index automation.
- Require every pass to exit successfully; parse the resulting PDF with lopdf
  inside the resource-limited sandbox and require at least one page. Read logs
  with a bound, report truncation, and never reuse intermediate outputs.

## Tests and production gates

Implement red-green tests for path/size/collision validation, exact explicit
engine selection and disabled shell/rc arguments. Trusted opt-in native tests
must verify basic PDFs for all three engines, multi-file input, missing tools,
invalid LaTeX, nonzero exit with a PDF, timeout and cancellation cleanup.
Run controlled isolation probes inside the sandbox (never hostile host TeX):
no host marker visibility, no inherited secret env, no host network namespace,
read-only input/system paths and bounded tmpfs. Process-tree tests must confirm
no descendants remain after timeout/cancellation.

Bubblewrap is a mechanism, not a complete sandbox policy. This profile has no
seccomp filter or aggregate cgroup memory/CPU/PID enforcement. RLIMIT_AS is not
an aggregate memory limit; per-process CPU is not an aggregate CPU quota. Keep
this worker experimental and do not expose compilation to untrusted clients
until cgroup/pid quotas, authentication, worker deployment separation, hostile
input review and full compatibility gates pass. CI's default fast tests must
not silently treat skipped native/isolation tests as evidence of readiness.
