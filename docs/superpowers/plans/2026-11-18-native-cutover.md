# Native migration completion

Execution: continue the approved native implementation plus independent review.
The user explicitly requested full completion without additional planning stops.

## Scope and sequencing

1. Establish compatibility and deployment gates on `feature/native-migration-completion`.
   Keep Python until these gates are green. Preserve legacy binary PDF contracts.
2. On a separate `chore/retire-python` branch from validated dev, delete the
   reference implementation and dependency/test/deployment remnants, update
   version/status/docs, and verify the resulting Python-free tree and image.
3. Use PRs into dev only. Preserve main, existing tags and rollback commit
   8f87470. Do not deploy public traffic or tag a stable release without evidence.

## Implementation tasks

- Tests first: PORT/bind configuration, token validation and protected routes;
  unauthenticated loopback remains compatible, remote listeners require a token.
- Expand trusted fixtures for custom class/package, resume/cover letter,
  tables/TikZ, uploaded OTF/TTF, icons, Unicode, BibTeX/Biber and missing assets.
  Compare reference and Rust PDF pages/text/fonts/visual rendering in the same
  native environment. Record timings without assuming a speedup.
- Preserve the IDE and upload the selected project, including binary fonts and
  images, with bounded traversal; honor explicitly selected compilers. Test the
  actual browser flow and authenticated API handling.
- Replace the Python socket isolation probe with a Rust-based probe.
- Add a pinned multi-stage Rust/TeX Docker build, non-root deployment, bounded
  tmpfs, CPU/memory/PID limits and explicit sandbox prerequisites. Add Compose,
  PORT support, readiness healthcheck and signal handling. Verify container
  compilation with all three engines and no Python interpreter in the image.
- Publish checked-in OpenAPI and Rust/JavaScript/Kotlin integration examples.
- Gate releases on Rust/native/container checks and dependency/license review.
- Update current guides rather than accumulating contradictory milestone prose.

## Verification

For behavior changes: reproduce a failing regression, implement, rerun the
targeted test, then full fmt/clippy/test/release and native suites. Full CI must
include latexmk/Biber and the container sandbox, with no ignored gate silently
skipped. Validate filesystem/resource/auth/ZIP regressions. Independently review
the complete diff and fix blockers before integration.

## Review focus

Token leakage through logs/worker environment; input preparation before limits;
namespace creation under container seccomp/AppArmor; uploaded font paths;
main-file-relative bibliography assets; process cleanup on termination; stale
Python commands/docs after removal; release artifact publication before gates.

## Honest completion boundary

Native Rust migration and Python removal are the immediate deliverable. Optional
WASM and further performance tuning remain capability-described follow-ups.
External client-owned templates or live release traffic cannot be invented;
record available protocol/fixture checks separately from live client validation.
