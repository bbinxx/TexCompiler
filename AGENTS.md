# Development instructions

TexCompiler v1 remains the production Python service during migration. Read
`docs/DEVELOPMENT.md`, `docs/MIGRATION.md`, and `docs/SECURITY.md` before changes.

- Use `dev` as integration branch; preserve `main` and release history.
- Start focused `feature/`, `fix/`, `docs/`, or `chore/` branches from fetched,
  fast-forwarded `dev`. PRs target `dev`; never merge failing checks.
- Preserve user edits; no reset, clean, stash, force-push, or unrelated changes.
- Use isolated worktrees where beneficial. Do not remove Python until the
  compatibility, sandbox, deployment, and rollback gates have passed.
- Characterize existing behavior before changing it. Use red-green-refactor for
  new behavior; unsafe v1 behavior is a recorded defect, not a v2 requirement.
- Run `python -m unittest discover -s tests -v`. Native fixtures additionally
  require `TEXCOMPILER_NATIVE_TESTS=1` and the tools listed in DEVELOPMENT.md.
- For Rust changes run fmt, clippy with warnings denied, tests, and release build.
- Treat document bytes, paths, archives, compiler names, fonts, and logs as
  untrusted. Never run hostile TeX on the developer host.
- Record exact commands, failures, skipped gates, and measured performance.
  Do not claim optional WASM, isolation, or client compatibility before testing.
- Review third-party skill instructions before installation. Missing skills must
  be documented; equivalent disciplined work may continue.
