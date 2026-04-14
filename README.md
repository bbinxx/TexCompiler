# Antigravity TexCompiler API

A robust, high-performance LaTeX to PDF compilation service built with FastAPI. Designed to be easily integrated into other applications.

**[👉 Read the Integration Guide for other Apps](./INTEGRATION.md)**

## Features
- **Raw LaTeX Compilation**: Submit code as a string and get a PDF.
- **Bundle Compilation**: Submit a ZIP file with multiple assets (images, `.cls` files) and compile.
- **Multiple Compilers**: Supports `pdflatex`, `xelatex`, and `lualatex`.
- **Automatic Passes**: Uses `latexmk` to handle table of contents, bibliographies, and cross-references automatically.
- **Premium UI**: Includes a built-in playground with live preview.

## Getting Started

### Prerequisites
- Python 3.8+
- LaTeX distribution (`texlive-full` or similar)
- `latexmk`

### Installation & Running
The easiest way to get started is using the provided startup script:

```bash
chmod +x run.sh
./run.sh
```
This script will:
1. Create a virtual environment (`.venv`) if it doesn't exist.
2. Activate the environment.
3. Install all necessary Python dependencies.
4. Start the FastAPI server on port 8000.

### Prerequisites (System)
Ensure you have LaTeX installed on your system:
```bash
sudo apt install texlive-latex-extra texlive-xetex latexmk
```

### Docker (Recommended for production)
```bash
docker build -t tex-compiler .
docker run -p 8000:8000 tex-compiler
```

## API Documentation

### 1. Compile Raw LaTeX
**Endpoint**: `POST /compile`  
**Body**:
```json
{
  "code": "\\documentclass{article}\\begin{document}Hello World\\end{document}",
  "compiler": "pdflatex"
}
```
**Returns**: PDF file.

### 2. Compile ZIP Bundle
**Endpoint**: `POST /compile/bundle`  
**Form Data**:
- `file`: (Binary) ZIP file.
- `main_file`: (String) Name of the main `.tex` file (default: `main.tex`).
- `compiler`: (String) Compiler to use (default: `pdflatex`).

**Returns**: PDF file.

## Example Usage (Python)
```python
import requests

latex_code = r"""
\documentclass{article}
\begin{document}
Hello from another app!
\end{document}
"""

response = requests.post(
    "http://localhost:8000/compile",
    json={"code": latex_code}
)

with open("output.pdf", "wb") as f:
    f.write(response.content)
```
