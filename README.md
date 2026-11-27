# TexCompiler v2

**Rust migration complete — `2.0.0-rc.1`, release candidate.** Axum/Tokio serves
the browser IDE and compiles LaTeX through isolated native TeX Live workers.
Python application code, dependency manifests and launchers have been removed.
No Python interpreter is needed by the service or included in its runtime image.

## Choose how to run

| Method | Start command | Dependencies |
|---|---|---|
| Direct Cargo, no shell launcher | `cargo build --locked --bins` then `cargo run --locked --bin texcompiler` | Rust and native tools below |
| Release binaries, no shell launcher | `cargo build --locked --release --bins` then `./target/release/texcompiler` | Native tools below |
| Convenience shell launcher | `./run.sh` or `./run-rust.sh` | Same native dependencies; builds both binaries |
| Docker Compose | `docker compose up --build -d` | Docker Engine, Compose plugin and a compatible Linux host |

The HTTP binary automatically finds the worker beside itself. Always build or
install **both** binaries together. Native workers require Linux; Windows users
should use a Linux/WSL2 environment. macOS/Windows native TeX workers are not
implemented. Docker Desktop/WSL host namespace policies must be checked separately.

## Install

Clone the repository and work from its root:

```sh
git clone https://github.com/bbinxx/TexCompiler.git
cd TexCompiler
```

For native builds install [Rust with rustup](https://rust-lang.org/install), then:

```sh
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
```

Ubuntu 24.04 native tool and document dependencies:

```sh
sudo apt-get update
sudo apt-get install --no-install-recommends \
  ca-certificates curl build-essential pkg-config meson ninja-build libcap-dev \
  xz-utils bubblewrap util-linux texlive-latex-extra texlive-xetex texlive-luatex \
  texlive-fonts-recommended texlive-bibtex-extra texlive-science \
  latexmk biber ghostscript poppler-utils ttf-bitstream-vera
```

Ubuntu's bundled Bubblewrap may be too old. **Version 0.13.1 is the validated
profile**; follow [the installation guide](docs/INSTALLATION.md) for its
checksum-verified build and host-policy prerequisites. Use Docker Compose if
you prefer the image to supply those dependencies. Ubuntu's TeX packages may
install Python for optional tools; TexCompiler itself uses no Python environment.

Arch Linux native dependencies:

```sh
sudo pacman -S --needed base-devel rustup bubblewrap util-linux \
  texlive-latexextra texlive-xetex texlive-luatex texlive-fontsrecommended \
  texlive-bibtexextra texlive-science texlive-binextra biber ghostscript \
  poppler ttf-bitstream-vera
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
```

For containers install [Docker Engine](https://docs.docker.com/engine/install/)
and its [Compose plugin](https://docs.docker.com/compose/install/linux/).
You do not need Rust or TeX installed on the host for Docker builds.

## Run directly without shell launchers

Linux prerequisites: the pinned Rust toolchain, Bubblewrap 0.13.1, util-linux
`prlimit`, TeX Live with pdfLaTeX/XeLaTeX/LuaLaTeX, latexmk and BibTeX/Biber.

```sh
cargo build --locked --bins
cargo run --locked --bin texcompiler
```

Open **http://127.0.0.1:8001** and click **Compile**. Stop with Ctrl+C.
Loopback development needs no token. For optimized binaries:

```sh
cargo build --locked --release --bins
./target/release/texcompiler
```

For a different local port:

```sh
TEXCOMPILER_BIND=127.0.0.1:9000 ./target/release/texcompiler
```

## Run with the shell launchers

```sh
./run.sh
# Equivalent:
./run-rust.sh
```

These are optional wrappers around the build-and-run commands above. The
canonical helper is `scripts/dev/run-rust.sh`; both root launchers delegate to it.

## Test an HTTP compilation

```sh
curl --fail-with-body http://127.0.0.1:8001/compile \
  -H 'Content-Type: application/json' \
  -d '{"code":"\\documentclass{article}\\begin{document}Hello Rust!\\end{document}","compiler":"pdflatex"}' \
  -o document.pdf
```

## Run with Docker Compose

Install Docker Engine, Buildx and the Compose plugin on a Linux host that
supports the [worker policy](deploy/README.md). The image supplies Rust,
TeX Live and Bubblewrap. Run these commands from the repository root.

```sh
docker compose version
docker compose up --build -d --wait
curl --fail http://127.0.0.1:8001/ready
```

Open **http://127.0.0.1:8001**, and click
**Compile** to render the sample. See the [Docker deployment guide](docs/DEPLOYMENT.md#docker-compose)
for persistent configuration, an authenticated PDF request, and troubleshooting.

If you configure a token, enter it in the IDE's Settings or send `Authorization: Bearer TOKEN`.
The token stays in the browser tab's session. Keep it on your server or trusted
gateway; do not embed a shared service token in a public frontend bundle.
Unset or empty tokens allow anyone who can reach the API to compile. Configured tokens must be 32–256 non-whitespace ASCII characters.

Compose applies a non-root UID, read-only root, bounded temporary storage,
CPU/memory/PID limits and a namespace-compatible seccomp policy. Worker network,
filesystem and environment isolation remain enabled. Unsupported host policies
fail closed; privileged containers and shell escape are not required.

Use `TEXCOMPILER_TEX_PROFILE=full` for additional TeX fonts/languages. This image
is larger; test your own templates. Python-dependent external helpers and
shell-escape workflows are unsupported. Hosted platforms must support the
documented namespace/container policy; the old Render blueprint was removed.

Useful container commands:

```sh
docker compose logs -f
docker compose ps
docker compose down
# Build the broader package profile:
TEXCOMPILER_TEX_PROFILE=full docker compose up --build -d --wait
```

For plain `docker build`/`docker run`, systemd installation, a platform PORT,
reverse-proxy/TLS setup, upgrades and rollback, see
[DEPLOYMENT.md](docs/DEPLOYMENT.md). Container security settings are required;
plain `docker run IMAGE` does not configure the worker's host prerequisites.

## Features and API

- pdfLaTeX, XeLaTeX and LuaLaTeX; explicit compiler choice or conservative detection.
- latexmk multipass and BibTeX/Biber workflows; three direct passes if absent.
- JSON projects and ZIP bundles, custom classes/packages, fonts and image assets.
- Existing editor, file explorer, local persistence, whole-project uploads,
  diagnostics, shortcuts and PDF preview.
- Private workspaces, disabled shell escape, bounded concurrency, timeouts and
  cleanup when calls are cancelled.

| Method | Route | Result |
|---|---|---|
| POST | /compile | Source or multi-file JSON -> binary PDF / JSON error |
| POST | /compile/bundle | Multipart ZIP project -> binary PDF / JSON error |
| POST | /detect-compiler | Compiler recommendation |
| GET | /compilers | Available engines |
| GET | /health | Liveness and compilation status |
| GET | /ready | 200 when startup compilation succeeds; otherwise 503 |

Versioned compile/project/engine/health/capability aliases are documented in
[API.md](docs/API.md). See [client examples](docs/CLIENTS.md) for server-side
JavaScript and Kotlin integration. Existing request models and binary responses
remain compatible; unsafe compiler/path/shell-escape quirks are intentionally
rejected. Remote clients must configure authentication.

| Setting | Default | Purpose |
|---|---|---|
| TEXCOMPILER_BIND | 127.0.0.1:8001 | Explicit numeric listener; overrides PORT |
| PORT | unset | Bind 0.0.0.0 to platform port |
| TEXCOMPILER_API_TOKEN | unset | Optional bearer token for compilation |
| TEXCOMPILER_WORKER | Sibling binary | Trusted same-build helper path |
| RUST_LOG | info | Log filter; tokens/source are not logged |

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
# Full equipped Linux gate; requires all native prerequisites.
cargo test --locked --all-features -- --include-ignored
node --test tests/web/*.test.mjs
# With a running service, Node 22+ and Chrome:
node scripts/verification/browser.cjs
```

CI verifies native engines, bibliography/fonts, namespace/resource cleanup,
the Python-free restricted container and rendered browser PDFs. Seven legacy
HTTP comparisons match text, pages, font names and page renderings; see
[migration evidence](docs/MIGRATION.md). Measurements are recorded without a
speedup claim. User-owned templates and live external deployments need their
own acceptance checks before a stable release.

## Structure and status

```text
src/                   Rust HTTP service, library and isolated worker
web/                   Browser IDE templates and assets
tests/                 Rust integration tests and browser-module tests
fixtures/              Trusted TeX compatibility inputs
scripts/dev/           Development launcher
scripts/verification/  Browser, container and migration verification
deploy/                Container security policy and license notice
docs/                  Current guides and migration evidence
.github/workflows/     Checks required before integration/release
```

Optional WASM, persistent cache, asynchronous job APIs and further performance
tuning are not implemented; capabilities describe only available functionality.
`production_ready` remains false pending the stable-release security and
real-client acceptance review. No public deployment or stable tag is created
by this source migration. Python rollback source remains in Git history.

Read [development](docs/DEVELOPMENT.md), [architecture](docs/ARCHITECTURE.md),
[security](docs/SECURITY.md), [deployment](docs/DEPLOYMENT.md),
[migration/rollback](docs/MIGRATION.md), [benchmarks](docs/BENCHMARKS.md),
[structure](docs/STRUCTURE.md), [versioning](docs/VERSIONING.md) and
[changelog](CHANGELOG.md).
