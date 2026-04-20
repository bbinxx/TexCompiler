# TexCompiler

A high-performance LaTeX to PDF compilation service with a built-in web IDE, workspace management, and auto font path resolution.

## Features

- **Web IDE** — built-in code editor with syntax highlighting, PDF preview, and build console
- **Workspace Management** — create and manage LaTeX projects directly in the browser (like Photopea)
- **File Explorer** — sidebar file tree with create/rename/delete, folder organization, image upload
- **Multi-Compiler** — pdflatex, xelatex, lualatex + auto-detection of the best engine from code content
- **Auto Font Resolution** — hardcoded `\setmainfont{...}[Path = ...]` paths are automatically corrected to match the current system
- **latexmk** — automatic multi-pass compilation for cross-references and bibliographies
- **Cross-Platform** — runs on Linux, macOS, and Windows (native .bat + cross-shell .sh)

## Quick Start

```bash
# Linux / macOS / Git Bash
./run.sh

# Windows (cmd.exe)
run.bat
```

Opens at `http://localhost:8000`.

## Workspace Workflow

1. Open the app — the file explorer sidebar shows your workspace projects
2. Click **"+ New"** to create a project (subfolder with a `main.tex` sample)
3. Click a project to browse its files in the sidebar
4. Click any `.tex` file to open it in the editor
5. Edit freely — files auto-save before compile
6. Press **Ctrl+Enter** or click **Compile** to generate the PDF
7. Right-click files/folders for rename and delete

Projects live on disk under `./workspace/` (configurable via `WORKSPACE_DIR` env).

## API

| Endpoint | Method | Description |
|---|---|---|
| `/workspace/projects` | GET | List projects |
| `/workspace/projects` | POST | Create a project |
| `/workspace/projects/{name}/files` | GET | List files in a project |
| `/workspace/projects/{name}/files/{path}` | GET | Read file content |
| `/workspace/projects/{name}/files/{path}` | PUT | Write file content |
| `/workspace/projects/{name}/files/{path}` | POST | Create empty file |
| `/workspace/projects/{name}/files/{path}` | DELETE | Delete file/folder |
| `/workspace/projects/{name}/rename` | POST | Rename a file or folder |
| `/workspace/projects/{name}/upload` | POST | Upload a file (multipart) |
| `/workspace/projects/{name}/compile` | POST | Compile project |
| `/workspace/projects/{name}/pdf` | GET | Get compiled PDF |
| `/compile` | POST | Compile raw LaTeX code |
| `/compile/bundle` | POST | Compile a ZIP bundle |
| `/compilers` | GET | List available compilers |
| `/detect-compiler` | POST | Detect best compiler for code |
| `/health` | GET | Server status |

## Deploy

```bash
docker build -t texcompiler .
docker run -p 8000:8000 texcompiler
```

Set `WORKSPACE_DIR` env to a persistent volume for project data.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8000` | Server port |
| `WORKSPACE_DIR` | `./workspace` | Projects storage directory |
| `COMPILE_TIMEOUT` | `120` | Compilation timeout (seconds) |
| `LOG_LEVEL` | `INFO` | Logging level |
