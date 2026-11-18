# Migration status and gates

## Versioning

This work is the start of the major v2 migration. The Rust foundation is
`2.0.0-dev.1` (unreleased), while Python remains the production entry point.
Stable `2.0.0` requires the gates below. See [VERSIONING.md](VERSIONING.md) and
[CHANGELOG.md](../CHANGELOG.md) for milestone and release handling.

## Current state

Phase 0 is partial: source/runtime audit, contract characterization, trusted
basic and multi-file fixtures, security findings and local timing samples.
Python production code, Docker entry point, browser assets and runtime
requirements are unchanged. A parallel Rust health/capabilities foundation is implemented; no Rust compilation
or WASM backend is implemented.

Audited tracked Python/config/deployment/release/browser sources at 54c5d89.
No pre-existing tests or AGENTS/DEVELOPMENT guidance existed. Git origin HEAD
already points to dev; main is preserved. dev was fetched and fast-forwarded
(no new commits), then a dedicated audit worktree/branch was created.

## Remaining gates

1. Phase 0: representative real resume/cover-letter/class/package/font/icon,
   TikZ/table/bibliography/Unicode/invalid/large-document fixtures; visual/font
   comparison references; real HTTP load, memory/CPU and container benchmarks;
   latexmk/Biber integration; reproducible Python/TeX environment. Request
   client-owned templates only if unavailable locally; do not invent parity.
2. Phase 1: reviewed contracts and architecture; parallel Rust health/router
   skeleton, pinned Rust toolchain, fmt/clippy/test/release CI and dependency /
   license policy. Keep legacy routes served by Python until implemented.
3. Phases 2–4: legacy adapters, safe engine selection, native process isolation,
   multipass/log/PDF checks, project/ZIP/font handling, fixtures and red-green
   regressions. Explicit-engine behavior differs from v1; document the change.
4. Phases 5–6: bounded jobs/cancel/cache/metrics, equivalent-environment timing,
   API differential suite, real-client and browser compatibility verification.
5. Phase 7: switch production only after all supported fixtures pass text/page/
   font/visual checks, security gates pass and deployment/rollback are tested.
6. Phase 8: independently research/prototype optional WASM; never block native
   compatibility on it or label an untested backend production-ready.
7. Phases 9–10: actual Vitae Forge/TexSpace contract checks, examples, reviewed
   security/performance/deployment evidence, then stable release tag.

Each change uses a small branch and PR into dev. The first PR is this audit;
the next introduces a parallel Rust foundation. Never merge failing work or
remove Python merely to complete a milestone.

## Rollback

Retain Python files and existing release history during migration. Before any
production switch, publish immutable old/new image digests, record configuration
and API routing, and rehearse restoring traffic to the previous Python image.
Preserve project inputs; isolate or invalidate Rust caches by environment
fingerprint. No rollback drill or production switch has occurred yet.

## Foundation progress

The approved foundation plan introduces only GET /api/v1/health and
GET /api/v1/capabilities on a separate loopback listener. Six Rust tests cover
truthful readiness, backend inventory, unknown/legacy routes and bind parsing.
Rust toolchain/dependency lock and fmt/clippy/test/release CI are present.
This completes the initial foundation slice, not all of Phase 1 or Phase 0.
The worker security specification is the next reviewed deliverable; no TeX
execution endpoint should be added before its enforceable isolation design.
