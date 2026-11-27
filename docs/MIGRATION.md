# Rust migration status and rollback

## Native cutover — 2.0.0-rc.1

The application has migrated from FastAPI/Python to Rust/Axum/Tokio. Both HTTP
handling and native compilation orchestration are Rust. The default Dockerfile,
Compose profile, Procfile and launchers start Rust. Python application source,
dependency manifests, reference tests/benchmark launcher and Python CI are
removed. No Python interpreter is present in the assembled runtime image.

This is a release candidate, not a claim of public hostile-document or every-
client/template acceptance. No stable tag or public traffic change was made.
Optional WASM/cache/job APIs and performance tuning are separate follow-ups.

## Evidence

- PRs 1–5 established the audit, Rust API/worker, working IDE and clean layout.
- The native cutover PR adds remote authentication, PORT/readiness, input
  admission retained through cancellation, full-project IDE uploads and native
  filename preservation, then validates the container before retiring Python.
- evidence/native-comparison.json records seven same-host HTTP comparisons.
  Page counts, extracted text, font names and every page's 72 dpi rendering
  match the retained reference. Timings are sequential single observations;
  Rust adds isolation/multipass work, so no speedup is claimed.
- Rust fixtures additionally cover uploaded OTF/TTF, Unicode, custom classes/
  packages, TikZ/tables, missing resources, invalid source and bibliography.
- Native CI includes all ignored tests with latexmk/BibTeX/Biber. Container CI
  tests the assembled Python-free, non-root/read-only/resource/namespace profile
  with real PDFs and browser rendering. Rust fmt/Clippy/test/release checks pass
  before integration. The RustSec snapshot is recorded under evidence/.
- evidence/rollback.json records local routing Rust → Python → Rust with real
  PDFs at each step. It does not claim rollback of an external production image.

## Intentional differences

Unsafe shell escape, arbitrary compiler execution, unconfined paths/archives,
ignored exit codes and host font-path rewriting are not preserved. Explicit
engines are honored. Authentication is optional on all listeners. API.md documents
the maintained request/response contract and limits. The browser framework and
client data models were not rewritten.

## Rollback source and actual deployments

The last reference source is commit 8f87470ab4f9c47458fd0a217199617a0478a784.
Recover it in a separate worktree, never by resetting or overwriting current work:

```sh
git worktree add -b fix/controlled-legacy-rollback /tmp/texcompiler-legacy-rollback 8f87470ab4f9c47458fd0a217199617a0478a784
```

That source retains known v1 vulnerabilities and should only be used for a
controlled trusted-input rollback window. The new repository does not keep a
second Python application. For real deployments record immutable old/new image
digests and secret/configuration references, start the candidate separately,
test your own corpus, then switch routing. Retain the previous deployment
during observation and restore its routing/digest if acceptance fails.

## Remaining release acceptance

The local TexSpace request model and binary response handling match the API;
no live mobile/web client build was executed. Client-owned Vitae Forge templates
were unavailable locally. Validate those actual inputs and external deployment
before a stable release. The systemd template requires host-specific validation.
Full CTAN/external-command compatibility is not promised; Python/shell-escape
helpers are intentionally outside policy. production_ready stays false pending
the final security/client acceptance review.

The implementation remains native-first and independently reversible through
reviewed Git history. All ordinary changes use task branches and PRs into dev;
main and previous tags remain intact. No failing checks are merged to finish a
phase, and benchmark numbers are never inferred from the language choice.
