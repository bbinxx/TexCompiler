# Changelog

Entries describe implemented changes. Planned work lives in
[the migration roadmap](docs/MIGRATION.md). Version and release rules are in
[VERSIONING.md](docs/VERSIONING.md).

## [Unreleased] — 2.0.0-dev.2 experimental native worker

- Rust now compiles trusted native fixtures through a private Linux Bubblewrap
  worker using explicit pdfLaTeX, XeLaTeX and LuaLaTeX engines.
- Added byte-preserving project validation, private workspaces, bounded input /
  output / tmpfs / diagnostics, fixed compiler arguments, PDF parsing/page count,
  engine metadata, bounded worker slots, timeout and cancellation cleanup.
- Shell escape and latexmk rc files are disabled. latexmk is used when present;
  the direct fallback runs three passes and does not automate bibliographies.
- Added native/isolation regression tests and a separate native integration CI
  gate with all three engines and latexmk.
- This is an unreleased internal development milestone. HTTP compilation still
  runs on Python; aggregate quotas, authentication, full compatibility and WASM
  remain incomplete. No stable v2 tag, deployment or speedup is claimed.

## Development foundation — 2.0.0-dev.1 (unreleased)

This is the start of a **major version migration**, not a published stable
release. `2.0.0-dev.1` was the initial Rust package version; no matching release
tag has been created. Python remains the production compilation entry point.

### Added

- Migration audit, legacy API/OpenAPI snapshot, security findings, architecture,
  development workflow and compatibility gates ([PR #1](https://github.com/bbinxx/TexCompiler/pull/1)).
- Ten Python characterization/native tests, basic and multi-file fixtures,
  PDF text/page checks, raw local benchmark samples and Python contract CI.
- Parallel Axum/Tokio foundation, typed health and capability responses,
  validated numeric bind configuration and tracing ([PR #2](https://github.com/bbinxx/TexCompiler/pull/2)).
- Pinned Rust toolchain, dependency lockfile, six Rust integration tests and
  formatting, Clippy, test and release-build CI.

### Compatibility and limitations

- Legacy compilation endpoints, browser assets and Docker entry point remain
  on Python. Rust reports `compilation_ready=false` and no available backends.
- Native Rust compilation, worker isolation, font/project handling, jobs,
  caching, WASM and full client compatibility validation remain incomplete.
- Existing security findings remain open; this milestone is not a hardened
  compilation service. See [SECURITY.md](docs/SECURITY.md).
- No Rust speedup is claimed. Current measurements cover only the Python
  baseline; broader performance and compatibility gates remain open.

## Existing v1 releases

Historical tags through `v1.3.1` are preserved. The current Python source reports
`1.3.0`; this differs from the latest tag and must be reconciled explicitly
before the next v1 release. This changelog does not reconstruct or alter
historical release notes.
