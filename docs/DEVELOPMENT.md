# Development

Production remains Python/FastAPI, version 1.3.0. The parallel Rust foundation is development-only; compilation remains Python.

## Setup and verification

Use Python 3.11+ in a virtual environment. Install `requirements-dev.txt`.
Runtime dependencies remain unchanged; HTTPX is only a test dependency.

```sh
python -m pip install -r requirements-dev.txt
python -m unittest discover -s tests -v
TEXCOMPILER_NATIVE_TESTS=1 python -m unittest discover -s tests -v
PYTHONPATH=. python scripts/benchmark_python.py > docs/python-baseline.json
```

Native tests execute only repository-owned trusted fixtures and require
pdflatex, xelatex, lualatex, pdfinfo, and pdftotext. They validate PDF framing,
page count, extracted text, and both JSON and ZIP projects. The regular suite
uses no TeX execution except attempting a deliberately nonexistent executable.
The two mocked subprocess tests cover timeout translation and the v1 exit-code
defect; they do not validate sandboxing or process-tree termination.

The current host has TeX Live 2026/Arch Linux; latexmk and biber are absent.
Install latexmk and biber in the full integration environment. Package and
font snapshots are not pinned today. Do not run untrusted documents using v1.

## Workflow

Inspect status and diffs, fetch origin, fast-forward dev, create one focused
branch, implement tests before behavior, verify, review, and open a PR to dev.
Integrate only with green required checks, then fast-forward local dev and
remove the task branch/worktree after verified integration. Keep main and tags.
A clean audit worktree was created at `/tmp/texcompiler-phase0` on
`docs/migration-architecture`; original checkout stays on dev.

## Skills observed on 2026-11-18

Installed and read: writing-plans, using-git-worktrees,
test-driven-development (including writing-good-tests),
verification-before-completion. The full using-superpowers/brainstorming,
executing-plans/subagent-driven-development, finishing-a-development-branch,
Rust, and dedicated security-review/code-review packages were not found in
available catalogs/local skills. No unreviewed third-party skills were
installed. Use the available Superpowers-derived practices and explicit manual
security review. Review and install remaining guidance before Rust work.

New FastAPI/Starlette on this host emits a TestClient HTTPX deprecation warning;
the suite passes. Runtime requirements use minimum versions, not pins. A locked
Python test environment remains a follow-up reproducibility task.

## Rust foundation

`rust-toolchain.toml` pins Rust 1.100.0 and Cargo.lock locks dependency resolution.
Run `cargo run --locked` for a separate listener at 127.0.0.1:8001. Override with
`TEXCOMPILER_BIND=127.0.0.1:9001`; numeric IPv4/IPv6 socket addresses are accepted,
invalid/empty values fail startup. Python PORT does not configure this listener.
RUST_LOG controls tracing (default info). Stop using the normal process signal.
No compilation worker, IDE, legacy routes or authentication run in the HTTP binary.
A separate experimental worker helper now exists; see below.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
```

Foundation TDD evidence: empty router returned 404 for both requested endpoints;
the config test failed to import the missing parser. After implementation six
Rust integration tests pass. CI runs the same checks separately from Python.
No compilation performance change is claimed. Dependency/license audit tooling,
generated OpenAPI and production deployment validation remain future gates.

Rust-specific guidance was subsequently located and read upstream at
https://github.com/leonardomso/rust-skills/blob/master/SKILL.md (v1.5.1), without
installing third-party files. Applied typed boundary parsing, propagated errors,
no unsafe code, no user-input panics, and async route tests. Axum/Tower usage
was checked against their primary docs at https://docs.rs/axum/0.8.9/axum/ and
https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html.

## Experimental native worker

Current Rust milestone: `2.0.0-dev.2` (unreleased). Use the library's
`NativeCompiler::new(trusted_worker_binary)` with `Project::new` byte assets and
an explicit `Engine`; do not take executable paths from client requests. The
helper binary is `texcompiler-worker`; `cargo run --locked` still starts the API.
Run the helper only through the supervisor's private sandbox. It refuses normal
unsandboxed startup and is not a standalone service or security boundary by itself.

Linux prerequisites: Bubblewrap with --disable-userns and --size, permitted
unprivileged user namespaces, /usr/bin/prlimit, TeX engines under /usr/bin and
latexmk for automated bibliography/multipass workflows. No fallback bypasses
failed isolation. Local testing uses Bubblewrap 0.13.1, util-linux 2.42.4 and
TeX Live 2026/Arch Linux. System libpaper configuration must be visible read-only
for XeTeX's PDF driver. The sandbox discovers only the explicitly listed system
paths; TeX installed under /usr/local/texlive is not currently supported.

```sh
# Fast tests; native/isolation tests are intentionally ignored here.
cargo test --locked --all-features
# Full native gate: requires latexmk, all three engines, Python 3 for a probe.
cargo test --locked --all-features -- --include-ignored
# Local direct fallback gate when latexmk is absent (not a full native gate).
cargo test --locked --all-features -- --include-ignored --skip latexmk_all_engines_and_nested_main_are_verified
```

Native CI installs the dependencies and runs the full no-skip gate separately
from fast Rust/Python checks. Tests use trusted documents and controlled probes
inside the sandbox. No untrusted host TeX execution is allowed. Tests probe host
marker/env/network invisibility, read-only mounts, aggregate output tmpfs size,
and externally observed descendant/workspace/slot cleanup after timeout/cancel.
See native-worker-contract.md and SECURITY.md for production limitations.

The native CI fixture profile uses Ubuntu's canonical HTTPS mirrors and disables
optional apt recommendations after the first regional-mirror download exceeded
the job deadline before any tests ran. It explicitly installs all three engines,
latexmk, recommended TeX fonts and PDF tools. Biber/full compatibility corpus
coverage remains a future gate; this CI package profile does not define or shrink
the production runtime's maximum-compatibility installation.
