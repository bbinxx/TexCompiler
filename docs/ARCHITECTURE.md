# Rust migration architecture (proposed)

Keep FastAPI production and the IDE unchanged during incremental migration.
Add a parallel Rust binary/library using Axum, Tokio, Serde, tracing and
OpenAPI generation. Native TeX Live plus latexmk is authoritative. Do not
rewrite TeX or client document generation. Rust API and TeX environment are
separate operational concerns; TeX Live still dominates runtime footprint.

HTTP adapters translate legacy contracts into typed engine/project requests.
Versioned routes expose explicit engine/backend selection, jobs and capabilities.
Workspace validation owns normalized relative paths, bounded archive extraction,
and asset bytes. The engine adapter receives only validated workspaces and
allowlisted options. Workers own process groups, cancellation, quotas and
cleanup. PDF validation and diagnostics precede response delivery.

Default shell escape off. Production execution requires an enforced OS/container
sandbox with no network, a read-only TeX tree, private writable job directory,
restricted environment, CPU/memory/pid/disk limits and tenant authorization.
A temporary folder alone is not sufficient. Do not advertise readiness when
these boundaries or required engine dependencies are unavailable.

Use deterministic project-relative fonts without source rewriting. Track the
TeX distribution/package snapshot, engine version, font bytes and flags. Cache
keys hash all input bytes plus that environment fingerprint, compilation options
and authorization partition. No cross-tenant cache reuse by default. Add queue,
cache and deduplication only after baseline measurements and isolation tests.

Optional WASM is a later experimental adapter. Evaluate BusyTeX and alternatives
against fixtures, runtime requirements, package/font support and license terms.
Emscripten output may need an isolated JS worker; do not assume WASI support.
No engine or runtime has been researched/selected in this audit. Native remains
the fallback; syntax errors must not cause blind retries.

Planned Rust modules: api, models, config, compiler/native, compiler/detection,
workspace, security, diagnostics; add fonts/cache/jobs/metrics when their
focused tasks need them. Preserve existing static/ and templates/ paths.
