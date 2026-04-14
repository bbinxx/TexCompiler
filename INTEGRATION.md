# TexCompiler Integration Guide

This guide provides everything you need to know to integrate the **TexCompiler** into your own applications.

## 🚀 API Base URL
If running locally: `http://localhost:8000`

---

## 1. Multi-file JSON API (Recommended)
This is the most powerful endpoint. It allows you to send the main LaTeX code and all associated assets (images, fonts, classes) in a single JSON request.

**Endpoint**: `POST /compile`  
**Content-Type**: `application/json`

### Payload Structure
```json
{
  "main_file": "main.tex",
  "compiler": "pdflatex",
  "files": {
    "main.tex": "raw latex code string",
    "logo.png": "base64_encoded_string",
    "style.cls": "text_content_of_class_file"
  }
}
```

### 🐍 Python Example (using `requests`)
```python
import requests
import base64

def compile_resume(tex_content, profile_image_path):
    # Encode image to base64
    with open(profile_image_path, "rb") as img_file:
        img_base64 = base64.b64encode(img_file.read()).decode('utf-8')

    payload = {
        "main_file": "resume.tex",
        "compiler": "xelatex", # Better fonts support
        "files": {
            "resume.tex": tex_content,
            "profile.jpg": img_base64
        }
    }

    response = requests.post("http://localhost:8000/compile", json=payload)

    if response.status_code == 200:
        with open("output.pdf", "wb") as f:
            f.write(response.content)
        print("✅ Success: PDF saved to output.pdf")
    else:
        error_data = response.json()
        print(f"❌ Error: {error_data['error']}")
        print(f"Logs: {error_data['log']}")
```

### 🌐 JavaScript Example (Node.js/Browser fetch)
```javascript
async function requestPdf(latex, imageBase64) {
    const payload = {
        main_file: "document.tex",
        files: {
            "document.tex": latex,
            "logo.png": imageBase64
        }
    };

    const response = await fetch('http://localhost:8000/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    if (response.ok) {
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        window.open(url); // Opens generated PDF
    } else {
        const err = await response.json();
        console.error("Compilation failed:", err.log);
    }
}
```

---

## 2. ZIP Bundle API
Use this if you already have a structured project folder.

**Endpoint**: `POST /compile/bundle`  
**Content-Type**: `multipart/form-data`

### Parameters
- `file`: The ZIP archive.
- `main_file`: (Optional) Path to the primary `.tex` file (default: `main.tex`).
- `compiler`: (Optional) `pdflatex`, `xelatex`, or `lualatex`.

---

## 🛠️ Error Handling
The API is designed to return detailed feedback when things go wrong.

| Status Code | Meaning | Response Body |
| :--- | :--- | :--- |
| **200** | Success | Binary PDF Data |
| **400** | LaTeX Syntax Error | `{"error": "Compilation failed", "log": "..."}` |
| **500** | Server Crash | `{"error": "Internal Server Error", "detail": "..."}` |

**Pro Tip**: Always check the `log` field on a `400` error. It contains the raw output from the LaTeX compiler, which tells you exactly which line of LaTeX code has the syntax error.

---

## ⚙️ Supported Compilers
- **`pdflatex`**: Fast, standard, works for 90% of documents.
- **`xelatex`**: Best for custom system fonts and Unicode/UTF-8 support.
- **`lualatex`**: Modern successor, similar to xelatex but supports Lua scripting.

---

## 🌍 Deployment

### 🚀 Render.com (Recommended)
This repo includes a `render.yaml` and `Dockerfile`. Render is the best choice because it supports Docker, allowing you to run the full LaTeX suite.
1. Connect your GitHub/GitLab repo to Render.
2. Render will automatically detect the `render.yaml` and deploy using Docker.
3. **Note**: TeX builds are memory-intensive. For large documents, you may need a Render plan with at least 1GB or 2GB of RAM.

### 🍱 General Cloud (Heroku, Railway, DigitalOcean)
Use the included `Dockerfile` and `Procfile`. Most modern platforms will see these and handle the OS-level LaTeX installation for you automatically.

### ☁️ Netlify / Vercel
Standard serverless functions (Netlify Functions) **do not support** custom binaries like `pdflatex`. To use this compiler with a Netlify project:
1. Deploy the compiler to **Render** or **Railway**.
2. Call the Render/Railway API from your Netlify application.

---

## 🏥 Health Check
You can monitor the service at `/health`. It returns:
```json
{
  "status": "healthy",
  "pdflatex": "/usr/bin/pdflatex",
  "latexmk": "/usr/bin/latexmk"
}
```
