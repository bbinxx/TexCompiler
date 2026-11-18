# Security audit — v1, 2026-11-18

This is an audit, not a hardened deployment. Do not expose the current service
to untrusted documents. Findings below come from source inspection unless a
test is named; no malicious TeX or filesystem escape payload was executed.

| Priority | Finding and location | Required gate |
|---|---|---|
| Critical | compiler.py build_compile_cmd always enables shell escape | Disable it; require separately authorized isolated capability for external commands |
| Critical | routes.py joins user files/main_file to temporary root without confinement | Reject absolute paths, parents, drive prefixes, symlinks and path collisions; prove writes remain inside root |
| Critical | compiler.py fallback executes supplied compiler name/path | Allowlist installed engine identifiers and fixed executable paths |
| High | TeX inherits service filesystem, network and environment | Separate worker boundary; read-only TeX distribution, private workspace, no network or credentials |
| High | ZIP unpack_archive has no explicit entry/size/count policy | Validate members before writing, reject links/duplicates/escapes and decompression bombs; underlying ZIP sanitization is not a complete policy |
| High | No CPU/memory/disk/output limits, queue or upload bounds | Enforce container/process limits and bounded queue/concurrency |
| High | subprocess.run blocks async route; timeout only handles direct process | Async orchestration; terminate/reap process group on timeout/cancel; test descendants |
| High | runner ignores exit code and accepts any existing PDF | Check exit status and parse PDF; characterized in test_nonzero_exit_with_pdf_is_currently_success |
| High | No service authentication or tenant isolation; broad CORS | Authenticate at deployment and partition job/cache access by principal |
| Medium | exceptions/logs/health expose internal paths and text | Redact diagnostics, bound logs, separate liveness from readiness |
| Medium | font rewriting uses host paths and silently drops missing paths | Preserve source; resolve only approved project/system font resources |
| Medium | Docker runs as root, mutable apt/Python dependencies, broad COPY | Non-root runtime, pinned environment, .dockerignore, read-only root, private tmp |

TemporaryDirectory provides unique per-request directories and normal cleanup,
but is not a sandbox and cannot undo writes outside its root. No cache exists.
No symlink policy, output validation, tenant boundaries or process-tree cleanup
is implemented. subprocess receives an argument array (no shell=True), but that
does not make arbitrary executable selection or TeX shell escape safe.

The launchers also kill processes listening on PORT; do not use them in shared
production environments. Existing release workflow publishes on tags without
test gates. Security regression tests must use controlled fixtures and isolated
workers; never launch host exploit samples to demonstrate these findings.

## Experimental Rust worker boundary

The `2.0.0-dev.2` library/helper addresses workspace validation, fixed engine
arguments, no shell escape, namespace/file/env/network separation, bounded
output/tmpfs and kill/reap cleanup for its experimental execution path. It does
not remediate the still-production Python execution path. Rust compile routes are now enabled for local/trusted use in dev.3 after a
native startup probe; production_ready remains false.

[The native worker contract](native-worker-contract.md) lists mounts and limits.
Bubblewrap is not a complete security policy: aggregate cgroup memory/CPU/PID
limits, seccomp, authenticated tenant boundaries, deployment separation and
hostile-document review remain required before public use. Per-process
RLIMIT_AS/CPU limits do not prevent aggregate process exhaustion. Production readiness must stay false while these gates remain open.
The user explicitly prioritized a usable local service before further hardening;
this does not make the prototype suitable for public/untrusted deployments.
