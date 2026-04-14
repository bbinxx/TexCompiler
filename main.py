import os
import subprocess
import shutil
import tempfile
from pathlib import Path
from typing import Optional
from fastapi import FastAPI, HTTPException, UploadFile, File, Form, Request, Response
from fastapi.responses import FileResponse, JSONResponse, HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import logging
import traceback

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="Antigravity TexCompiler API",
    description="A high-performance LaTeX to PDF compilation service.",
    version="1.0.0"
)

@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(f"Global exception caught: {exc}")
    logger.error(traceback.format_exc())
    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal Server Error",
            "detail": str(exc),
            "trace": traceback.format_exc() if app.debug else "Enable debug for trace"
        }
    )

# Mount static files
app.mount("/static", StaticFiles(directory="static"), name="static")
templates = Jinja2Templates(directory="templates")

# Enable CORS for all origins (useful for cross-app usage)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

import base64

class CompileRequest(BaseModel):
    code: Optional[str] = None
    files: Optional[dict[str, str]] = None # filename -> content (text or base64)
    main_file: str = "document.tex"
    compiler: str = "pdflatex"

@app.get("/", response_class=HTMLResponse)
async def read_root(request: Request):
    """
    Returns the premium LaTeX editor UI.
    """
    return templates.TemplateResponse(request=request, name="index.html")

def run_compilation(workdir: Path, main_file: str, compiler: str = "pdflatex"):
    """
    Runs latexmk to compile the document.
    """
    cmd = [
        "latexmk",
        f"-{compiler}",
        "-pdf",
        "-interaction=nonstopmode",
        "-shell-escape",  # Allow some packages to run external commands if needed
        main_file
    ]
    
    try:
        process = subprocess.run(
            cmd,
            cwd=workdir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60  # Timeout after 60 seconds
        )
        
        if process.returncode != 0:
            # Look for the log file
            log_path = workdir / main_file.replace(".tex", ".log")
            log_content = ""
            if log_path.exists():
                log_content = log_path.read_text(errors="replace")
            
            return False, log_content or process.stdout or process.stderr
        
        return True, None
    except subprocess.TimeoutExpired:
        return False, "Compilation timed out (max 60 seconds)."
    except Exception as e:
        return False, str(e)

@app.post("/compile")
async def compile_tex(request: CompileRequest):
    """
    Compiles LaTeX. Supports:
    1. Single 'code' string.
    2. 'files' dictionary (filename -> content/base64).
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        
        main_file = request.main_file
        
        # Handle files dictionary
        if request.files:
            for filename, content in request.files.items():
                file_path = tmp_path / filename
                file_path.parent.mkdir(parents=True, exist_ok=True)
                
                # Check if it's base64 (for images/assets)
                # Heuristic: try to decode if extension is binary-friendly
                binary_exts = {'.png', '.jpg', '.jpeg', '.pdf', '.cls', '.sty', '.zip'}
                is_binary = any(filename.lower().endswith(ext) for ext in binary_exts)
                
                try:
                    if is_binary:
                        try:
                            decoded = base64.b64decode(content)
                            with open(file_path, "wb") as f:
                                f.write(decoded)
                            continue
                        except:
                            pass
                    
                    # Default to text
                    file_path.write_text(content)
                except Exception as e:
                    logger.error(f"Error writing file {filename}: {e}")
        else:
            # Single file mode
            if not request.code:
                return JSONResponse(status_code=400, content={"error": "No code or files provided."})
            tex_file = tmp_path / "document.tex"
            tex_file.write_text(request.code)
            main_file = "document.tex"
        
        success, error_log = run_compilation(tmp_path, main_file, request.compiler)
        
        if not success:
            return JSONResponse(
                status_code=400,
                content={"error": "Compilation failed", "log": error_log}
            )
        
        pdf_name = main_file.replace(".tex", ".pdf")
        pdf_path = tmp_path / pdf_name
        
        if not pdf_path.exists():
            return JSONResponse(
                status_code=500, 
                content={"error": "PDF not generated despite success.", "log": error_log}
            )

        # Read into memory to allow tmpdir to be cleaned up
        pdf_content = pdf_path.read_bytes()
        
        from fastapi import Response
        return Response(
            content=pdf_content, 
            media_type='application/pdf',
            headers={"Content-Disposition": f"attachment; filename={pdf_name}"}
        )

@app.post("/compile/bundle")
async def compile_bundle(
    file: UploadFile = File(...),
    main_file: str = Form("main.tex"),
    compiler: str = Form("pdflatex")
):
    """
    Upload a ZIP bundle with assets and compile the main file.
    """
    if not file.filename.endswith(".zip"):
        raise HTTPException(status_code=400, detail="Only ZIP bundles are supported.")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        # Save zip
        zip_path = tmp_path / "bundle.zip"
        with open(zip_path, "wb") as f:
            shutil.copyfileobj(file.file, f)
            
        # Unpack
        try:
            shutil.unpack_archive(str(zip_path), extract_dir=str(tmp_path))
        except Exception as e:
            return JSONResponse(status_code=400, content={"error": f"Failed to unpack ZIP: {str(e)}"})
            
        # Check if main file exists
        if not (tmp_path / main_file).exists():
            return JSONResponse(
                status_code=400, 
                content={"error": f"Main file '{main_file}' not found in bundle."}
            )
            
        success, error_log = run_compilation(tmp_path, main_file, compiler)
        
        if not success:
            return JSONResponse(
                status_code=400,
                content={"error": "Compilation failed", "log": error_log}
            )
            
        pdf_name = main_file.replace(".tex", ".pdf")
        pdf_path = tmp_path / pdf_name
        
        if not pdf_path.exists():
            return JSONResponse(status_code=500, content={"error": "PDF not generated.", "log": error_log})
            
        # Read into memory
        pdf_content = pdf_path.read_bytes()
        
        from fastapi import Response
        return Response(
            content=pdf_content,
            media_type='application/pdf',
            headers={"Content-Disposition": f"attachment; filename={pdf_name}"}
        )

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
