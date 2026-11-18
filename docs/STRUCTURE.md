# Repository structure

| Area | Ownership |
|---|---|
| src/main.rs | Configuration, listener and signal handling |
| src/bin/texcompiler-worker.rs | Internal worker entry point |
| src/api/ | HTTP routes, auth/admission, JSON/ZIP adaptation and embedded web delivery |
| src/compiler/ | Engines/detection, supervisor, sandbox, worker and typed protocol |
| src/config/ | Validated bind/PORT/token configuration |
| src/workspace/ | Input policy and private regular-file workspaces |
| web/ | Existing IDE templates, modules and styles |
| tests/ | Rust contract/native/security tests; tests/web contains Node module tests |
| fixtures/ | Trusted compatibility sources and project assets |
| scripts/dev/ | Optional development launcher |
| scripts/verification/ | Browser/container and migration evidence tooling |
| deploy/ | Container policy, systemd template and environment example |
| docs/ | Current guides, evidence and clearly historical plans |
| .github/workflows/ | Fast/native/container/dependency and gated release checks |

Cargo.toml/Cargo.lock and platform-standard Dockerfile/Compose/Procfile remain
at the root. run.sh and run-rust.sh are optional Rust launchers. The HTTP binary
also runs directly after cargo build --bins; there is no launcher requirement.

This retirement branch removes the legacy Python subtree, dependency files, CI and launchers. Merge follows container validation.
JSON observations and plans remain evidence; they are not live runtime code.
Do not add empty speculative crates/modules or reintroduce Python orchestration.
Preserve shared fixtures and HTTP /static URLs when reorganizing source files.
