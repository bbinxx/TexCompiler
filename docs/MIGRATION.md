# Migration status and gates

## Native cutover candidate

The completion branch adds authenticated remote listeners, PORT support,
readiness, bounded input preparation through cancellation, project uploads in
the IDE, basename-preserving native jobs, and an isolated Rust container profile.
Seven trusted Python/Rust HTTP fixtures match page counts, extracted text, font
names and every page's 72 dpi rendering. Exact observations are recorded in
[native-comparison.json](evidence/native-comparison.json); these are not speedup
claims. Rust-only tests additionally cover uploaded OTF/TTF and missing resources.

Dockerfile.rust and compose.yaml are the cutover candidates. CI must prove the
final runtime has no Python interpreter, enforces its configured resource and
filesystem policy, and compiles all engines/fonts/BibTeX/Biber plus browser PDFs
before the separate Python retirement PR. The old Dockerfile remains unchanged
until that gate passes. No public deployment or stable release is claimed.

## Versioning

This work is the start of the major v2 migration. The Rust foundation is
`2.0.0-dev.3` (unreleased), while Python remains the production entry point.
Stable `2.0.0` requires the gates below. See [VERSIONING.md](VERSIONING.md) and
[CHANGELOG.md](../CHANGELOG.md) for milestone and release handling.

## Current state

Phase 0 is partial: source/runtime audit, contract characterization, trusted
basic and multi-file fixtures, security findings and local timing samples.
Python production code, Docker entry point and runtime requirements remain
available for comparison and rollback. The Rust service now provides native
HTTP compilation and the existing browser IDE. WASM remains unimplemented.

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
The worker security specification and experimental implementation are described
below. Public TeX execution endpoints remain gated on production isolation.

## Experimental native worker milestone — 2.0.0-dev.2

Rust now validates in-memory byte projects and compiles trusted fixtures using
pdfLaTeX/XeLaTeX/LuaLaTeX inside Bubblewrap, with fixed arguments, shell escape
and latexmk rc loading disabled, bounded protocol/log/PDF output, PDF parsing,
page counts, version metadata, a supervisor deadline and two slots per compiler
instance. Cancellation retains the private workspace until process reaping.
See [the worker contract](native-worker-contract.md) for exact limits.

This is a Rust library/helper, not a production switch or public compile API.
Fast tests cover path/count/byte/collision validation and protocol boundaries;
opt-in tests cover native documents, controlled isolation and process cleanup.
The native CI job explicitly includes ignored tests, installs latexmk and all
three engines, and must pass before integration. Local direct-pass testing does
not prove latexmk compatibility. The full project/font/ZIP APIs, aggregate
cgroup/PID limits, seccomp, authentication and broad compatibility remain gates.

## Working service milestone — 2.0.0-dev.3

Following the user's instruction to make the service functional before further
fine tuning, Rust now serves the existing IDE plus /compile, /compile/bundle,
/detect-compiler, /compilers and /health. Versioned compile/project/engine/health/
capability aliases are available. Source, JSON multi-file and multipart ZIP
requests produce native PDF responses. Explicit compilers are honored; omitted
or blank selections auto-detect from source. Assets stay byte-preserving and
ZIP entries are validated in memory before private workspace creation.

Start with `./run-rust.sh` (Linux; defaults to 127.0.0.1:8001). It builds both
Rust binaries; no Python interpreter is used by this service. The API probes
native compilation at startup and reports compilation_ready independently of
production_ready (still false). Only use trusted documents on loopback while
aggregate quotas/authentication and broad compatibility work continue.

Verified locally: HTTP PDFs for all three engines, JSON/ZIP multi-file projects,
compiler error diagnostics and the actual Chrome IDE compile/preview flow.
The native latexmk CI gate is still required before integration. Existing
Python remains only until full replacement/compatibility and rollback gates pass.

## Python removal commitment

After full migration, remove legacy/python/app/, legacy/python/main.py, Python runtime and test dependency
manifests, Python-only launchers and Python CI, legacy/python/tests/test_legacy.py and
legacy/python/scripts/benchmark_python.py after their useful coverage has been ported. Replace the Docker/Procfile/
Render startup configurations with the validated Rust service and retain no
Python runtime dependency in the final image. Convert useful regression fixtures
and contracts to Rust before retiring the reference suite. Confirm client/API,
IDE, native engine/font/project, security and deployment parity first. Historical
Git tags/images provide rollback; Python must not remain in the final v2 runtime.

Python removal depends on native functional parity and a validated Rust
deployment, not on completing optional WASM, caching or performance fine tuning.
