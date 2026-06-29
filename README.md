# TexCompiler

A high-performance LaTeX to PDF compilation service with a browser-based IDE, local file system access, and auto font path resolution.

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

## Quick Start

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
