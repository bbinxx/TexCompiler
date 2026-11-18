# TexCompiler v2

**Major Rust migration — unreleased `2.0.0-dev.3`.** A working Axum/Tokio service
compiles native LaTeX and serves the existing browser IDE. Python is isolated in
`legacy/python/` for comparison and the existing deployment; it will be removed
completely after native functional parity and the Rust deployment are verified.

## Working features

- pdfLaTeX, XeLaTeX and LuaLaTeX; explicit engine selection and conservative auto-detection.
- Raw LaTeX, JSON multi-file projects and multipart ZIP bundles; binary PDF responses.
- Native latexmk workflows when installed, or a three-pass direct fallback.
- Existing source editor, local file explorer, build console and visible PDF preview.
- Private validated workspaces, disabled shell escape, namespace isolation,
  bounded output, timeouts and cancellation cleanup.

This is a local/trusted-document prototype. Production quotas, authentication,
complete client/template/font verification and deployment validation remain open.
WASM, job APIs, caching and performance tuning are follow-ups. No stable v2
release or measured Rust speedup is claimed.

## Run Rust

Linux prerequisites: Rust 1.100.0, Bubblewrap 0.13.1, util-linux `prlimit`, and
TeX Live with the engines you need. Install latexmk for bibliography automation.
A maximum-compatibility TeX Live installation is recommended for complex templates.

```sh
./run-rust.sh
```

Open **http://127.0.0.1:8001**, edit the demo and press **Compile**. The launcher
builds both Rust binaries. No Python interpreter is used by this service.

```sh
curl http://127.0.0.1:8001/compile \
  -H 'Content-Type: application/json' \
  -d '{"code":"\\documentclass{article}\\begin{document}Hello Rust\\end{document}","compiler":"pdflatex"}' \
  -o document.pdf
```

| Setting | Default | Purpose |
|---|---|---|
| TEXCOMPILER_BIND | 127.0.0.1:8001 | Numeric listener address |
| TEXCOMPILER_WORKER | Sibling worker binary | Trusted same-build helper path; launcher sets it |
| RUST_LOG | info | Rust logging filter |

The IDE still compiles its active source. Clients can send whole projects and
assets through the multi-file API; automatic IDE workspace upload is pending.

## API

| Method | Route | Result |
|---|---|---|
| POST | /compile | JSON source/files -> PDF or JSON error |
| POST | /compile/bundle | Multipart ZIP project -> PDF or JSON error |
| POST | /detect-compiler | Compiler recommendation |
| GET | /compilers | Installed allowlisted engines |
| GET | /health | Version, engines and functional readiness |

Versioned compile/project/engine/health/capability aliases are documented in
[API.md](docs/API.md). `compilation_ready` reflects the startup compile probe;
`production_ready` remains false while the production gates are open.

## Project layout

```text
src/                 Rust library, HTTP executable and worker executable
  api/               Routes, request adaptation and embedded web delivery
  compiler/          Engine selection, sandbox, supervisor and worker protocol
  config/            Validated runtime configuration
  workspace/         Validated project inputs and private workspaces
web/                 Shared IDE templates and static assets
tests/               Rust integration and contract tests
fixtures/            Shared trusted LaTeX compatibility inputs
scripts/dev/         Canonical Rust development launcher
scripts/verification/ Browser smoke test (Node + Chrome)
legacy/python/       Temporary Python reference service, tests and dependencies
docs/                Architecture, API, migration, security and development guides
.github/workflows/   Rust, native integration, legacy reference and release checks
```

See [STRUCTURE.md](docs/STRUCTURE.md) for ownership and migration cleanup rules.
Root launchers preserve familiar commands. Cargo and platform-required Docker,
Procfile and Render manifests stay at the root intentionally.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
# Equipped Linux runner: all native and isolation gates, without skips.
cargo test --locked --all-features -- --include-ignored
```

The actual Chrome editor -> compile -> rendered PDF flow is checked with:

```sh
TEXCOMPILER_URL=http://127.0.0.1:8001 node scripts/verification/browser.cjs
```

## Migration and deployment

The next cutover candidate is available through `compose.yaml` and
`Dockerfile.rust`; its full container CI gate must pass before Python retirement.
Remote listeners require `TEXCOMPILER_API_TOKEN` (32–256 printable ASCII characters).
The IDE accepts this token in Settings, storing it only for the tab session.
The selected workspace is now uploaded with its source, fonts and assets.
Explicit compiler selections are preserved in both the API and IDE.
See [runtime policy](deploy/README.md) before using the container candidate.

[PR #4](https://github.com/bbinxx/TexCompiler/pull/4) introduces the working Rust
service after the audit and API foundation. Stable v2 requires the remaining
[migration gates](docs/MIGRATION.md). Optional WASM/caching/tuning do not block
removing Python once native parity and deployment are validated.

The root Dockerfile, Procfile and Render deployment still start the **legacy
Python reference** until the Rust deployment cutover is verified. Reference
setup and environment variables are in [legacy/python/README.md](legacy/python/README.md).
Do not mistake that existing image for a completed Python-free v2 deployment.

Further guidance: [development](docs/DEVELOPMENT.md),
[architecture](docs/ARCHITECTURE.md), [security](docs/SECURITY.md),
[versioning](docs/VERSIONING.md), [benchmarks](docs/BENCHMARKS.md),
and [changelog](CHANGELOG.md).
