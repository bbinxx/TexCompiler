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
    title="TexCompiler API",
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

import re

def run_compilation(workdir: Path, main_file: str, compiler: str = "pdflatex"):
    """
    Runs latexmk to compile the document with detailed logging.
    """
    cmd = [
        "latexmk",
        f"-{compiler}",
        "-pdf",
        "-interaction=nonstopmode",
        "-shell-escape",
        main_file
    ]
    
    logger.info(f"🚀 Starting compilation: {' '.join(cmd)} in {workdir}")
    
    try:
        process = subprocess.run(
            cmd,
            cwd=workdir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60
        )
        
        # Look for the log file to extract details/warnings
        log_path = workdir / main_file.replace(".tex", ".log")
        log_content = ""
        warnings = []
        
        if log_path.exists():
            log_content = log_path.read_text(errors="replace")
            # Extract warnings (LaTeX warnings usually start with "LaTeX Warning:")
            warnings = re.findall(r"(?:LaTeX|Package|Class) Warning:.*?\n\n", log_content, re.DOTALL)
            if not warnings:
                # Fallback for simpler warnings
                warnings = [line.strip() for line in log_content.splitlines() if "Warning:" in line]

        if process.returncode != 0:
            logger.error(f"❌ Compilation failed with return code {process.returncode}")
            return False, log_content or process.stdout or process.stderr, warnings
        
        logger.info(f"✅ Compilation successful: {main_file}")
        return True, log_content, warnings
        
    except subprocess.TimeoutExpired:
        logger.error("⏱️ Compilation timed out (60s limit)")
        return False, "Compilation timed out (max 60 seconds).", []
    except Exception as e:
        logger.error(f"💥 Unexpected error during compilation: {str(e)}")
        return False, str(e), []

@app.post("/compile")
async def compile_tex(request: CompileRequest):
    """
    Compiles LaTeX with detailed response including warnings.
    """
    logger.info(f"📥 Received compile request (Compiler: {request.compiler})")
    
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        main_file = request.main_file
        
        if request.files:
            logger.info(f"📁 Processing {len(request.files)} files...")
            for filename, content in request.files.items():
                file_path = tmp_path / filename
                file_path.parent.mkdir(parents=True, exist_ok=True)
                
                binary_exts = {'.png', '.jpg', '.jpeg', '.pdf', '.cls', '.sty', '.zip'}
                is_binary = any(filename.lower().endswith(ext) for ext in binary_exts)
                
                try:
                    if is_binary:
                        decoded = base64.b64decode(content)
                        file_path.write_bytes(decoded)
                    else:
                        file_path.write_text(content)
                except Exception as e:
                    logger.warning(f"⚠️ Failed to write {filename}: {e}")
        else:
            if not request.code:
                return JSONResponse(status_code=400, content={"error": "No code provided"})
            (tmp_path / "document.tex").write_text(request.code)
            main_file = "document.tex"

        success, log, warnings = run_compilation(tmp_path, main_file, request.compiler)
        
        if not success:
            return JSONResponse(
                status_code=400,
                content={
                    "error": "Compilation failed",
                    "log": log,
                    "warnings": warnings,
                    "details": "Check the log for missing packages or syntax errors."
                }
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

@app.get("/health")
async def health_check():
    """
    Health check endpoint for deployment platforms.
    """
    try:
        # Check if pdflatex is available
        import shutil
        pdflatex_path = shutil.which("pdflatex")
        return {
            "status": "healthy",
            "pdflatex": pdflatex_path or "Not found",
            "latexmk": shutil.which("latexmk") or "Not found"
        }
    except Exception as e:
        return JSONResponse(status_code=500, content={"status": "unhealthy", "error": str(e)})

if __name__ == "__main__":
    import uvicorn
    # Use PORT from environment for deployment compatibility (Render, Heroku, etc.)
    port = int(os.environ.get("PORT", 8000))
    uvicorn.run(app, host="0.0.0.0", port=port)
