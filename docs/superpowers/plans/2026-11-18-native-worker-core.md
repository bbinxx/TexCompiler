# Native worker core implementation plan

> **For agentic workers:** Execute task-by-task using the native workflow and independent review. The executing-plans package is unavailable; use the installed TDD and verification skills.

**Goal:** Compile trusted fixtures using Rust's experimental isolated native worker without changing Python production traffic.

**Architecture:** Validate in-memory projects, create private workspaces, then supervise a same-build helper in Bubblewrap. TeX/PDF parsing stay in the resource-limited helper. No public compile endpoint until production isolation gates pass.

**Tech Stack:** Rust 1.100, Tokio process/time/sync/io, tempfile, Serde, base64, lopdf, Linux Bubblewrap/prlimit and installed TeX Live.

**Spec:** docs/native-worker-contract.md; original migration security/compatibility requirements in docs/MIGRATION.md.

## Global constraints

- Branch feature/native-worker-core from validated dev; PR targets dev.
- Keep Python, browser assets, runtime entry point and legacy contracts unchanged.
- Honor explicit engine selection; disable shell escape and latexmk rc files.
- Never bypass failed isolation. No untrusted host TeX tests.
- Keep compilation_ready=false; no production-ready or speedup claim.

## Review focus

- Invalid portable paths and file/directory collisions rejected before writes.
- Explicit engine retained even when source requires another engine.
- Missing sandbox/dependencies fail closed; no host fallback.
- Compiler nonzero exit/partial PDF must fail even when PDF exists.
- Wall timeout/cancel kills descendants and cleans private workspaces.

### Task 1: Typed project and engine inputs

Files: src/compiler/mod.rs, engine.rs, src/workspace.rs, tests/worker_inputs.rs.
Interfaces: Engine enum (PdfLatex/XeLatex/LuaLatex); Project::new(files,
main_file) -> Result<Project, InputError>; NativeCompiler::new(worker_executable).

- [ ] Write path traversal, Windows path, count/byte limit, missing main and
      collision tests; run them and observe missing implementation failures.
- [ ] Implement validation, private RAII workspace and exact engine arguments.
- [ ] Run tests; review path policy against all Review focus input classes.

### Task 2: Isolated native execution

Files: src/compiler/native.rs, sandbox.rs, worker.rs, src/main.rs;
tests/native_worker.rs and tests/sandbox.rs.
Interfaces: NativeCompiler::compile(Project, Engine) -> Result<Compilation,
NativeError> (async); worker main reads bounded request and writes bounded result.

- [ ] Test missing sandbox failures, timeout/cancellation and nonzero result
      rejection before implementing supervisor/helper behavior.
- [ ] Implement bounded process IO, timeout, fail-fast semaphore, private tmpfs,
      environment/mount policy and same-build helper entry mode.
- [ ] Test trusted basic/multi-file documents, invalid LaTeX and output validity
      on all engines; implement latexmk selection or documented three-pass fallback.
- [ ] Test controlled isolation probes and descendant cleanup. Record unsupported
      native environments explicitly; never silently downgrade sandbox policy.

### Task 3: Verification and review

Files: docs/DEVELOPMENT.md, MIGRATION.md, SECURITY.md, CHANGELOG.md and README.md.

- [ ] Run fmt, clippy -D warnings, cargo test --all-features and release build.
- [ ] Run full Python/native baseline and explicit Rust/native isolation tests.
- [ ] Record exact capabilities, remaining cgroup/auth/compatibility gates and
      engine/tool metadata; update documentation without claiming stable v2.
- [ ] Obtain independent code/security review; fix important findings, rerun
      affected checks, commit and open focused PR; merge only with green CI.

## Execution evidence

User approved native execution of this plan. Independent security specification
review completed before implementation. Input tests failed for missing modules;
native compilation then failed with Unavailable before the runner existed.
A diagnostics-bound test failed before its check was added. Local checks now
pass 29 Rust tests including trusted native/isolation gates; the one explicit
latexmk gate is not runnable locally because latexmk is absent. Native CI must
run all 30 tests without filtering before integration. The implementation uses
a separate src/bin/texcompiler-worker.rs rather than changing the HTTP main.
Independent code review found no critical/high defect; it requested the explicit
latexmk gate and a larger JSON envelope, both now implemented. Final CI remains
the integration gate, not a skipped-test claim of readiness.
