# Repository structure and ownership

The root is a single Rust application crate. Avoid empty future-feature modules
or extra crates until an implemented responsibility warrants them.

| Area | Responsibility |
|---|---|
| src/main.rs | HTTP process startup and listener configuration |
| src/bin/texcompiler-worker.rs | Internal sandbox helper entry point |
| src/api/routes.rs | Stateful HTTP routes and response mapping |
| src/api/project.rs | JSON/base64 and ZIP transport adaptation into validated projects |
| src/api/assets.rs | Embedded IDE/template/static delivery at stable HTTP URLs |
| src/compiler/ | Typed engines, detection, process supervision, sandbox and protocol |
| src/config/ | Validated environment/runtime configuration |
| src/workspace/ | Project path/size policy and private workspace lifecycle |
| web/static and web/templates | Shared browser IDE sources, preserved across backends |
| tests/ | Cargo-discovered Rust integration tests; worker unit tests stay near code |
| fixtures/ | Shared trusted document inputs; no generated PDFs or auxiliary output |
| scripts/dev/ | Rust launch/build helpers |
| scripts/verification/ | Optional browser verification tooling |
| legacy/python/ | Reference FastAPI service, dependency manifests, launchers, tests and benchmark |
| docs/ | Current architecture/API/security/status plus historical execution plans |
| .github/workflows/ | Separate fast/native/reference checks and release workflow |

Root run-rust.sh delegates to scripts/dev/run-rust.sh. Root run.sh/run.bat remain
compatibility wrappers for the legacy service. Platform-standard Dockerfile,
Procfile and render.yaml retain their expected root locations; they still use
legacy Python until deployment cutover. Their internal paths match the isolated
legacy directory. Shared HTTP /static URLs do not change when files move to web/.

Python tests are invoked explicitly with PYTHONPATH=legacy/python, so they do
not mix with Cargo's tests. Source, dependencies and helper scripts for Python
must stay under legacy/python while retained. Remove that complete subtree,
legacy root wrappers, legacy CI and Python deployment commands after native
parity/cutover. Preserve useful fixtures and version-neutral baseline records.
No Python application code should be added to the Rust runtime.
