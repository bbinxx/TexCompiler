# Development

TexCompiler is a single Rust application crate with two executables: the HTTP
service and its internal sandbox worker. No Python environment is needed.

## Setup

Install Rust through rustup; rust-toolchain.toml pins Rust 1.100.0, rustfmt and
Clippy. Cargo.lock locks dependencies. Linux native compilation additionally
needs Bubblewrap 0.13.1, permitted unprivileged user namespaces, prlimit, the
three TeX engines and their fonts. Install latexmk, BibTeX/Biber and Poppler
tools for the complete compatibility suite. See deploy/README.md for the
validated container policy. Non-Linux worker execution is unsupported.

Run ./run.sh or ./run-rust.sh from the repository; both build the two binaries
and start the service on loopback port 8001. cargo run alone does not guarantee
the helper is built; use cargo build --bins first when launching manually.
TEXCOMPILER_BIND overrides PORT. Setting PORT binds a remote listener and
requires TEXCOMPILER_API_TOKEN. Never use a client-supplied worker path.

## Workflow

Inspect status/diffs, preserve user changes, fetch origin and fast-forward dev.
Create a focused feature/, fix/, docs/ or chore/ branch from validated dev;
worktrees are useful for isolation. No ordinary commits directly to dev.
Write a failing regression before behavior changes, implement, verify, review,
then open a PR targeting dev. Merge only with green required checks. Preserve
main, tags and release history; no reset/clean/stash/force-push without authority.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
node --test tests/web/*.test.mjs
```

Fast Rust tests intentionally ignore native environment gates. On an equipped
Linux system run every test explicitly:

```sh
cargo test --locked --all-features -- --include-ignored
```

Do not call the full suite green if any native prerequisite is missing. Native
CI installs the complete fixture profile, including latexmk/Biber, and includes
all ignored tests. Host checks never run hostile TeX outside the sandbox.
The container job additionally runs all engines/fonts/bibliography and the IDE
under the exact Compose restrictions. It checks absence of Python interpreters.

With a running service, Node 22+ and Chrome:

```sh
TEXCOMPILER_URL=http://127.0.0.1:8001 node scripts/verification/browser.cjs
```

Set CHROME_BIN for a different Chrome path. Set TEXCOMPILER_API_TOKEN for an
authenticated service. The smoke test uses an isolated browser profile and
does not put tokens in the URL or print them. Frontend module tests use Node's
built-in runner, with no npm dependency installation or UI framework migration.

## Layout and dependencies

See STRUCTURE.md. Keep process execution, input validation and HTTP adaptation
separate. web/ assets are embedded at build time, so rebuild after UI edits.
TeX Live is the authoritative backend; do not rewrite TeX or patch user source
to hide missing fonts/packages. Copy uploaded fonts/assets without rewriting.

Use the checked-in toolchain and lock file. Review dependency changes, licenses
and RustSec advisories. CI installs pinned cargo-audit; do not ignore advisories
without a documented review. Docker pins base digests, the package manifest and
Bubblewrap checksum; rebuild/test explicitly when updating them. The build
package manifest is retained in /app for environment diagnosis.

## Skills and review

Use installed planning, worktree, TDD, debugging and verification skills where
applicable; request an independent review before integration. The initial
environment lacked full Superpowers, Rust and dedicated security-review
packages; that limitation is recorded in the historical plans. Equivalent
typed/error-propagating Rust guidance and explicit manual security review were
used. Do not install unreviewed third-party instructions automatically.

## Rollback and current limits

The legacy implementation is recoverable at commit 8f87470. The recorded local
traffic rehearsal switched Rust → Python → Rust successfully; no live production
traffic was changed. Follow MIGRATION.md before an actual rollout and retain
immutable deployed image digests. Optional WASM/cache/job APIs remain follow-ups;
the RC does not claim full CTAN, live-client or public hostile-document approval.
