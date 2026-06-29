import base64
import logging
import shutil
import tempfile
from pathlib import Path

from fastapi import APIRouter, Form, HTTPException, Request, Response, UploadFile, File
from fastapi.responses import JSONResponse

from app.config import APP_VERSION, BINARY_EXTENSIONS
from app.compiler import detect_compiler, list_available_compilers, resolve_font_paths, run_compilation
from app.models import CompileRequest, DetectCompilerRequest

logger = logging.getLogger(__name__)
router = APIRouter()


# ---- Original endpoints ----

@router.get("/", include_in_schema=False)
async def index(request: Request):
    templates = request.app.state.templates
    return templates.TemplateResponse(
        request=request, name="index.html", context={"version": APP_VERSION}
    )


@router.post("/compile")
async def compile_tex(request_data: CompileRequest):
    code = request_data.code or (
        "\n".join(request_data.files.values()) if request_data.files else ""
    )
    compiler = detect_compiler(code, request_data.compiler)
    logger.info("Compile request received (compiler=%s, files=%s)", compiler, len(request_data.files) if request_data.files else 0)

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        main_file = request_data.main_file

        if request_data.files:
            for filename, content in request_data.files.items():
                file_path = tmp_path / filename
                file_path.parent.mkdir(parents=True, exist_ok=True)

                is_binary = any(
                    filename.lower().endswith(ext) for ext in BINARY_EXTENSIONS
                )
                try:
                    if is_binary:
                        file_path.write_bytes(base64.b64decode(content))
                    else:
                        file_path.write_text(content)
                except Exception as exc:
                    logger.warning("Failed to write file %s: %s", filename, exc)
        else:
            if not request_data.code:
                return JSONResponse(
                    status_code=400,
                    content={"error": "No code provided"},
                )
            patched = resolve_font_paths(request_data.code)
            if patched != request_data.code:
                logger.info("Font paths resolved in LaTeX code")
            (tmp_path / "document.tex").write_text(patched)
            main_file = "document.tex"

        success, log, warnings = run_compilation(tmp_path, main_file, compiler)

        if not success:
            return JSONResponse(
                status_code=400,
                content={
                    "error": "Compilation failed",
                    "log": log,
                    "warnings": warnings,
                    "details": "Review the log for missing packages or syntax errors.",
                },
            )

        pdf_name = main_file.replace(".tex", ".pdf")
        pdf_path = tmp_path / pdf_name

        if not pdf_path.exists():
            return JSONResponse(
                status_code=500,
                content={
                    "error": "PDF was not generated despite successful compilation.",
                    "log": log,
                },
            )

        pdf_content = pdf_path.read_bytes()

    return Response(
        content=pdf_content,
        media_type="application/pdf",
        headers={"Content-Disposition": f"attachment; filename={pdf_name}"},
    )


@router.post("/compile/bundle")
async def compile_bundle(
    file: UploadFile = File(...),
    main_file: str = Form("main.tex"),
    compiler: str = Form("pdflatex"),
):
    if not file.filename or not file.filename.endswith(".zip"):
        raise HTTPException(status_code=400, detail="Only ZIP bundles are supported.")

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        zip_path = tmp_path / "bundle.zip"

        with open(zip_path, "wb") as f:
            shutil.copyfileobj(file.file, f)

        try:
            shutil.unpack_archive(str(zip_path), extract_dir=str(tmp_path))
        except Exception as exc:
            return JSONResponse(
                status_code=400,
                content={"error": f"Failed to unpack ZIP: {exc}"},
            )

        if not (tmp_path / main_file).exists():
            return JSONResponse(
                status_code=400,
                content={"error": f"Main file '{main_file}' not found in bundle."},
            )

        success, log, warnings = run_compilation(tmp_path, main_file, compiler)

        if not success:
            return JSONResponse(
                status_code=400,
                content={
                    "error": "Compilation failed",
                    "log": log,
                    "warnings": warnings,
                },
            )

        pdf_name = main_file.replace(".tex", ".pdf")
        pdf_path = tmp_path / pdf_name

        if not pdf_path.exists():
            return JSONResponse(
                status_code=500,
                content={"error": "PDF not generated.", "log": log},
            )

        pdf_content = pdf_path.read_bytes()

    return Response(
        content=pdf_content,
        media_type="application/pdf",
        headers={"Content-Disposition": f"attachment; filename={pdf_name}"},
    )


@router.post("/detect-compiler")
async def detect_compiler_endpoint(req: DetectCompilerRequest):
    compiler = detect_compiler(req.code, req.preferred)
    return {"compiler": compiler}


@router.get("/compilers")
async def get_compilers():
    return {"compilers": list_available_compilers()}


@router.get("/health")
async def health_check():
    try:
        compilers = list_available_compilers()
        return {
            "status": "healthy",
            "version": APP_VERSION,
            "compilers": compilers,
            "latexmk": shutil.which("latexmk") or "Not found",
        }
    except Exception as exc:
        return JSONResponse(
            status_code=500,
            content={"status": "unhealthy", "error": str(exc)},
        )
