# TexCompiler

A high-performance LaTeX to PDF compilation service with a browser-based IDE, local file system access, and auto font path resolution.

## TexCompiler v2 — major migration in development

**This repository has begun the major v2 migration from Python/FastAPI to
Rust/Axum. Stable v2 is not released.** The working experimental Rust service is versioned
`2.0.0-dev.3`; Rust now serves compilation and the browser IDE when launched with `run-rust.sh`.
The existing Python deployment remains available during verification.

| Component | Current status |
|---|---|
| Python compilation service | Existing runtime; reports `1.3.0` |
| Rust `2.0.0-dev.3` | Working native HTTP compilation, ZIP/multi-file API and browser IDE |
| Stable `2.0.0` | Planned; requires compatibility, security, deployment and rollback gates |

The first milestones are merged: [audit and compatibility baseline (PR #1)](https://github.com/bbinxx/TexCompiler/pull/1)
and [parallel Rust API foundation (PR #2)](https://github.com/bbinxx/TexCompiler/pull/2).
The implementation/runtime and deployment transition warrants a major version;
legacy API compatibility remains a requirement. Rust now has an experimental native worker for trusted fixtures on Linux.
The Rust HTTP service is available for local/trusted use. Production quotas,
broader compatibility and optional WASM remain pending.

See the [changelog](CHANGELOG.md), [versioning policy](docs/VERSIONING.md), and
[migration gates](docs/MIGRATION.md). The existing Docker deployment below still starts Python until migration gates
pass. Python will be removed completely after full migration.

## Features

- **Browser IDE** — Ace Editor with LaTeX syntax highlighting, PDF preview, and build console
- **Local Workspace** — pick any folder on your device as the workspace; files are read/written directly via the File System Access API (no server-side storage)
- **File Explorer** — sidebar tree with directory navigation, create/rename/delete/duplicate files
- **Image Insertion** — add images from your device or a URL; saved to your workspace and inserted as `\includegraphics{}`
- **Multi-Compiler** — pdflatex, xelatex, lualatex with automatic engine detection from code content
- **Auto Font Resolution** — font paths in `\setmainfont{...}[Path = ...]` are automatically corrected to match the system
- **latexmk** — automatic multi-pass compilation for cross-references and bibliographies
- **Session Persistence** — workspace folder handle is stored in IndexedDB and restored across page refreshes
- **Cross-Platform** — Linux, macOS, Windows (native `.bat` + cross-shell `.sh`)

## Run the Rust service

On Linux with Rust, Bubblewrap, util-linux and TeX Live installed:

```bash
./run-rust.sh
```

Open **http://127.0.0.1:8001**, edit the demo and press **Compile**. The service
supports pdfLaTeX, XeLaTeX, LuaLaTeX, multi-file JSON and ZIP bundles. It builds
both binaries automatically and does not use Python. Use latexmk when installed;
the fallback makes three engine passes. For another port, set
`TEXCOMPILER_BIND=127.0.0.1:9001`. This is an unreleased local/trusted-document
service; production hardening and compatibility verification continue.

## Legacy Python Quick Start

```bash
# Install LaTeX (Ubuntu/Debian)
sudo apt install texlive-full latexmk

# Install LaTeX (macOS)
brew install --cask mactex

# Run
./run.sh        # Linux / macOS / Git Bash
run.bat         # Windows (cmd.exe)
```

Opens at `http://localhost:8000`.

## Usage

1. Open the app, then click the **Settings** (gear) icon in the header
2. Click **Select Folder** to choose a workspace folder on your device
3. The sidebar file explorer shows the folder contents — navigate into subdirectories
4. Click any `.tex` file to open it in the editor
5. Edit freely — **Ctrl+S** or click the Save button to persist
6. Files auto-save before each compile
7. Press **Ctrl+Enter** or click **Compile** to generate the PDF
8. **Right-click** any file or click the **`…`** button for Rename / Duplicate / Delete
9. Click the **image icon** to insert images from your device or a URL

## API

The API can be used independently of the web IDE for headless compilation.

### `POST /compile`

Compile raw LaTeX code and receive a PDF.

**Request (JSON):**

```json
{
  "code": "\\documentclass{article}\\begin{document}Hello\\end{document}",
  "compiler": "pdflatex"
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `code` | string | — | LaTeX source code |
| `files` | object | `null` | Map of filename → content for multi-file projects |
| `main_file` | string | `"document.tex"` | Entry point when using `files` |
| `compiler` | string | `"pdflatex"` | Compiler engine |

**Response:** `application/pdf` binary stream on success, or `application/json` error on failure.

**Example (curl):**

```bash
curl -X POST http://localhost:8000/compile \
  -H "Content-Type: application/json" \
  -d '{"code":"\\documentclass{article}\\begin{document}Hello\\end{document}"}' \
  -o output.pdf
```

**Multi-file example (curl):**

```bash
curl -X POST http://localhost:8000/compile \
  -H "Content-Type: application/json" \
  -d '{
    "files": {
      "main.tex": "\\documentclass{article}\\input{body}\\end{document}",
      "body.tex": "Hello from body!"
    },
    "main_file": "main.tex",
    "compiler": "pdflatex"
  }' \
  -o output.pdf
```

**Python example:**

```python
import requests

res = requests.post("http://localhost:8000/compile", json={
    "code": r"\documentclass{article}\begin{document}Hello\end{document}",
    "compiler": "pdflatex",
})
with open("output.pdf", "wb") as f:
    f.write(res.content)
```

### `POST /compile/bundle`

Compile a ZIP bundle containing LaTeX sources.

**Request:** `multipart/form-data`

| Field | Type | Default | Description |
|---|---|---|---|
| `file` | file | — | ZIP archive containing LaTeX files |
| `main_file` | string | `"main.tex"` | Entry point within the archive |
| `compiler` | string | `"pdflatex"` | Compiler engine |

**Response:** `application/pdf` binary stream.

**Example (curl):**

```bash
zip bundle.zip main.tex
curl -X POST http://localhost:8000/compile/bundle \
  -F "file=@bundle.zip" \
  -F "main_file=main.tex" \
  -F "compiler=pdflatex" \
  -o output.pdf
```

### `GET /compilers`

List available compiler engines.

**Response:**

```json
{
  "compilers": ["pdflatex", "xelatex", "lualatex", "latex"]
}
```

### `POST /detect-compiler`

Auto-detect the best compiler for given LaTeX code.

**Request (JSON):**

```json
{
  "code": "\\documentclass{article}\\usepackage{fontspec}\\begin{document}Hello\\end{document}",
  "preferred": "pdflatex"
}
```

**Response:**

```json
{
  "compiler": "lualatex"
}
```

### `GET /health`

Server health and version info.

**Response:**

```json
{
  "status": "healthy",
  "version": "1.3.0",
  "compilers": ["pdflatex", "xelatex", "lualatex"],
  "latexmk": "/usr/bin/latexmk"
}
```

### `GET /`

Serves the web IDE.

## Deploy

```bash
docker build -t texcompiler .
docker run -p 8000:8000 texcompiler
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8000` | Server port |
| `COMPILE_TIMEOUT` | `120` | Compilation timeout (seconds) |
| `LOG_LEVEL` | `INFO` | Logging level |
| `CORS_ORIGINS` | `*` | CORS allowed origins |

## Rust migration status

Rust now serves native compilation and the existing IDE. See
[the migration gates](docs/MIGRATION.md), [development workflow](docs/DEVELOPMENT.md),
[API documentation](docs/API.md), and [security status](docs/SECURITY.md).
Python is retained for compatibility comparison and the current Docker startup;
it will be removed after full migration. Stable v2 and WASM are not released.
