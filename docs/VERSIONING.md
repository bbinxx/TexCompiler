# Versioning and releases

The Python-to-Rust rewrite is a major v2 migration. Cargo.toml is the runtime
version source; Cargo.lock must match. The IDE and health response use that
version. The native cutover is **2.0.0-rc.1**, not a stable 2.0.0 release.

The README/changelog describe implemented functionality and remaining release
acceptance. Do not mark optional WASM, cache, jobs, benchmarks or live client
validation complete merely because the Rust source migration is complete.

Use conventional commits and focused branches/PRs into dev. Preserve main,
historical releases and tags. Never commit ordinary changes directly to dev,
force-push a shared branch or merge failing checks to finish a milestone.

A release requires Rust format/Clippy/tests/release build, the full native
fixture/security suite, container/browser validation and current dependency
review. The release workflow must gate publication on those checks. For a stable
tag, also complete actual client/template and production acceptance, record
image digests, benchmark limitations and rollback configuration. The migration
does not itself publish a tag or alter the default/release branch strategy.

Prereleases use v2.0.0-rc.N tags only when deliberately released; stable uses
v2.0.0 after acceptance. Future incompatible protocol or deployment behavior
requires a documented major-version policy rather than silent changes.
