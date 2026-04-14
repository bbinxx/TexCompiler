# 🚀 TexCompiler API

A robust, high-performance LaTeX to PDF compilation service built with FastAPI. Designed to be easily integrated into any application, from simple scripts to complex resume builders.

![LaTeX Compilation Request Flow](/home/bbin/.gemini/antigravity/brain/a7d5ec8d-e813-43cf-b0f5-387898e0f667/tex_api_flow_1776157821565.png)

## ✨ Features
- **Raw LaTeX Compilation**: Submit code as a string and get a PDF.
- **Multi-File Support**: Send `.tex`, `.cls`, `.sty`, and images (Base64) in a single JSON.
- **Bundle Compilation**: Submit a ZIP file with multiple assets and compile.
- **Multiple Compilers**: Supports `pdflatex`, `xelatex`, and `lualatex`.
- **Automatic Passes**: Uses `latexmk` to handle TOC, bibliographies, and cross-references.
- **Premium UI**: Includes a built-in playground with live preview.

---

## 🛠️ Getting Started

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

### System Dependencies (Ubuntu/Debian)
```bash
sudo apt install texlive-latex-extra texlive-xetex latexmk
```

### Docker (Recommended for production)
```bash
docker build -t tex-compiler .
docker run -p 8000:8000 tex-compiler
```

---

## 🔌 API Documentation

### 🚀 API Base URL: `http://localhost:8000`

| Endpoint | Method | Input Type | Description |
| :--- | :--- | :--- | :--- |
| `/compile` | `POST` | `application/json` | Compile raw code or multiple files. |
| `/compile/bundle` | `POST` | `multipart/form-data` | Upload ZIP and compile. |
| `/health` | `GET` | `None` | Service health status. |

---

## 📖 API Usage Guide (All Cases)

### 1. Simple Compilation (Raw String)
Perfect for quick snippets or single-page documents.

**Python:**
```python
import requests

payload = {
    "code": r"\documentclass{article}\begin{document}Hello world from API\end{document}",
    "compiler": "pdflatex"
}

response = requests.post("http://localhost:8000/compile", json=payload)
with open("basic.pdf", "wb") as f:
    f.write(response.content)
```

### 2. Multi-File Compilation (JSON + Base64 Image)
Ideal for resumes or documents with logos and custom classes.

**Python:**
```python
import requests
import base64

# Encode your images to base64
with open("profile.jpg", "rb") as img_file:
    img_b64 = base64.b64encode(img_file.read()).decode()

payload = {
    "main_file": "main.tex",
    "compiler": "xelatex",
    "files": {
        "main.tex": r"""
\documentclass{article}
\usepackage{graphicx}
\begin{document}
\section{Profile}
\includegraphics[width=0.3\textwidth]{profile.jpg}
\end{document}
""",
        "profile.jpg": img_b64
    }
}

response = requests.post("http://localhost:8000/compile", json=payload)
if response.status_code == 200:
    with open("resume.pdf", "wb") as f:
        f.write(response.content)
```

**JavaScript (Fetch):**
```javascript
async function compileWithImage(latex, imageBase64) {
    const payload = {
        main_file: "main.tex",
        files: {
            "main.tex": latex,
            "logo.png": imageBase64
        }
    };

    const res = await fetch('http://localhost:8000/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    if (res.ok) {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        window.open(url);
    }
}
```

### 3. ZIP Bundle Compilation
Best for complex projects that already exist as a folder.

**Python:**
```python
import requests

files = {'file': open('project.zip', 'rb')}
data = {'main_file': 'main.tex', 'compiler': 'pdflatex'}

response = requests.post("http://localhost:8000/compile/bundle", files=files, data=data)
with open("bundle_output.pdf", "wb") as f:
    f.write(response.content)
```

---

## 🛠️ Error Handling
The API returns detailed feedback on compilation failures.

| Status Code | Meaning | Response Body |
| :--- | :--- | :--- |
| **200** | Success | Binary PDF Data |
| **400** | LaTeX Syntax Error | `{"error": "Compilation failed", "log": "...", "warnings": []}` |
| **500** | Server Error | `{"error": "Internal Server Error", "detail": "..."}` |

---

## 🌍 Deployment

### Render.com (Recommended)
This repo includes a `render.yaml` and `Dockerfile`. Render automatically detects these.
1. Connect your repo to Render.
2. It will deploy using the Docker environment.
3. **Note**: TeX builds are memory-intensive. Use a plan with ≥ 1GB RAM.

---

## 🏥 Health Check
Monitor the service at `/health`.
```json
{
  "status": "healthy",
  "pdflatex": "/usr/bin/pdflatex",
  "latexmk": "/usr/bin/latexmk"
}
```
