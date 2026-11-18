# Changelog

Entries describe implemented changes. Planned work lives in
[the migration roadmap](docs/MIGRATION.md). Version and release rules are in
[VERSIONING.md](docs/VERSIONING.md).

## [Unreleased] — 2.0.0-dev.1 development milestone

This is the start of a **major version migration**, not a published stable
release. `2.0.0-dev.1` is the current Rust package version; no matching release
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
