# Rust foundation implementation plan

> **For agentic workers:** Use executing-plans task-by-task; an equivalent native workflow is required while that skill is unavailable. Steps use checkbox syntax.

**Goal:** Add a tested parallel Rust API foundation without changing Python production behavior.

**Architecture:** Axum router/library plus Tokio binary, initially health and capability inventory only. Keep legacy compilation on Python until its own security and compatibility gates pass.

**Tech Stack:** Rust stable (pin the installed validated release), Axum, Tokio, Serde, tracing, tower test utilities.

**Spec:** docs/MIGRATION.md and docs/ARCHITECTURE.md, reflecting the user migration requirements.

## Global constraints

- Native TeX Live remains authoritative; never remove Python before parity gates.
- One focused branch/PR from validated dev per deliverable. PRs target dev.
- No untrusted host TeX execution, default shell escape, or arbitrary executable names.
- Preserve legacy binary/error contracts; explicit engines must be honored in v2.
- Do not claim unimplemented endpoints or backends as capabilities.

## Review focus

- Missing engines: readiness reports unavailable compilation instead of healthy capacity.
- Explicit engine conflicts: requests never silently switch engines.
- Traversal/archive entries: rejected before writing any file outside a private root.
- Cancellation/timeout: descendants are terminated and workspace cleaned.
- Partial PDFs/nonzero exit: never returned as successful compilation.

### Task 1: Parallel Rust foundation (next PR)

Files: Cargo.toml, Cargo.lock, rust-toolchain.toml, src/lib.rs, src/main.rs,
src/api/mod.rs, src/config/mod.rs, tests/api_foundation.rs,
.github/workflows/rust.yml, docs/DEVELOPMENT.md.
Interfaces: `pub fn router() -> axum::Router`; versioned health returns
`{status:"starting",compilation_ready:false}` until an isolated worker exists.
Legacy endpoints stay on Python; do not redirect production traffic.

- [ ] Review/install Rust guidance and approve this architecture and contract.
- [ ] Branch feature/rust-api-foundation from integrated audit dev.
- [ ] Write tests asserting GET /api/v1/health is 200 with compilation_ready=false,
      unknown route is 404, and capabilities do not advertise native/WASM readiness.
- [ ] Run cargo test and observe missing router/route failures.
- [ ] Implement minimal router, typed responses and env-validated bind address.
- [ ] Add invalid bind configuration test before implementing parsing errors.
- [ ] Run fmt --check, clippy --all-targets --all-features -- -D warnings,
      cargo test --all-features and cargo build --release.
- [ ] Add pinned-toolchain CI with those commands; retain Python baseline CI.
- [ ] Review diff, document limitations, commit and open PR to dev.

### Task 2: Worker security specification (separate reviewed plan/PR)

Files: docs/SECURITY.md, docs/native-worker-contract.md, tests/security/.
Consumes: audit findings. Produces: documented worker request/result protocol
with validated relative paths, engine enum, bounded limits and cancellation.

- [ ] Select an enforceable deployment isolation mechanism and quotas; enumerate
      trusted TeX read paths, blocked network/env and private write mounts.
- [ ] Define tests for each Review focus condition, including descendant kill,
      disk exhaustion, symlinks, Windows-style paths and malformed archives.
- [ ] Specify legacy explicit-engine compatibility change and diagnostic limits.
- [ ] Review the specification before adding any endpoint that executes TeX.

### Later subsystem plans

Create independently reviewed plans for legacy adapters/native workers,
projects/fonts, jobs/cache/metrics, differential compatibility/deployment,
optional WASM research and final client/release validation. The phase gates in
MIGRATION.md bound those plans; this foundation does not implement them.
