import base64
import logging
import os
import shutil
import tempfile
from pathlib import Path

from fastapi import APIRouter, Form, HTTPException, Request, Response, UploadFile, File
from fastapi.responses import JSONResponse

from app.config import APP_VERSION, BINARY_EXTENSIONS, WORKSPACE_DIR
from app.compiler import detect_compiler, list_available_compilers, resolve_font_paths, run_compilation
from app.models import (
    CompileRequest,
    CreateFileRequest,
    CreateFolderRequest,
    CreateProjectRequest,
    DetectCompilerRequest,
    RenameRequest,
)

logger = logging.getLogger(__name__)
router = APIRouter()

WORKSPACE = Path(WORKSPACE_DIR)


def _ensure_workspace():
    WORKSPACE.mkdir(parents=True, exist_ok=True)


def _project_path(name: str) -> Path:
    p = (WORKSPACE / name).resolve()
    if not str(p).startswith(str(WORKSPACE.resolve())):
        raise HTTPException(status_code=403, detail="Path traversal denied")
    return p


def _file_path(project: str, file_rel: str) -> Path:
    p = (_project_path(project) / file_rel).resolve()
    if not str(p).startswith(str(WORKSPACE.resolve())):
        raise HTTPException(status_code=403, detail="Path traversal denied")
    return p


def _scan_files(dir_path: Path, prefix: str = "") -> list[dict]:
    items: list[dict] = []
    for entry in sorted(dir_path.iterdir(), key=lambda e: (not e.is_dir(), e.name.lower())):
        rel = f"{prefix}/{entry.name}" if prefix else entry.name
        if entry.is_dir():
            items.append({"name": entry.name, "path": rel, "type": "dir", "size": 0})
            items.extend(_scan_files(entry, rel))
        elif entry.is_file():
            items.append({"name": entry.name, "path": rel, "type": "file", "size": entry.stat().st_size})
    return items


# ---- Workspace ----

@router.get("/workspace/projects")
async def list_projects():
    _ensure_workspace()
    projects = []
    for entry in sorted(WORKSPACE.iterdir()):
        if entry.is_dir() and not entry.name.startswith("."):
            tex_files = list(entry.glob("*.tex"))
            projects.append({
                "name": entry.name,
                "files": len(tex_files),
                "has_pdf": (entry / "document.pdf").exists(),
            })
    return {"projects": projects, "workspace": str(WORKSPACE)}


@router.post("/workspace/projects")
async def create_project(req: CreateProjectRequest):
    _ensure_workspace()
    name = req.name.strip().replace(" ", "_").replace("/", "")
    if not name:
        raise HTTPException(status_code=400, detail="Invalid project name")
    path = _project_path(name)
    if path.exists():
        raise HTTPException(status_code=409, detail="Project already exists")
    path.mkdir(parents=True)
    sample = path / "main.tex"
    sample.write_text(
        "\\documentclass{article}\n"
        "\\begin{document}\n"
        f"Hello from {name}!\n"
        "\\end{document}\n"
    )
    return {"name": name, "path": str(path)}


@router.delete("/workspace/projects/{project}")
async def delete_project(project: str):
    path = _project_path(project)
    if not path.exists():
        raise HTTPException(status_code=404, detail="Project not found")
    shutil.rmtree(path)
    return {"deleted": project}


# ---- Project files ----

@router.get("/workspace/projects/{project}/files")
async def list_files(project: str):
    path = _project_path(project)
    if not path.exists():
        raise HTTPException(status_code=404, detail="Project not found")
    files = _scan_files(path)
    return {"files": files}


@router.get("/workspace/projects/{project}/files/{path:path}")
async def read_file(project: str, path: str):
    file = _file_path(project, path)
    if not file.exists() or not file.is_file():
        raise HTTPException(status_code=404, detail="File not found")
    if any(file.suffix.lower() == ext for ext in BINARY_EXTENSIONS):
        return Response(content=file.read_bytes(), media_type="application/octet-stream")
    return Response(content=file.read_text(errors="replace"), media_type="text/plain; charset=utf-8")


@router.put("/workspace/projects/{project}/files/{path:path}")
async def write_file(project: str, path: str, req: CreateFileRequest):
    file = _file_path(project, path)
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_text(req.content)
    return {"written": path}


@router.post("/workspace/projects/{project}/files/{path:path}")
async def create_file(project: str, path: str):
    file = _file_path(project, path)
    if file.exists():
        raise HTTPException(status_code=409, detail="File already exists")
    file.parent.mkdir(parents=True, exist_ok=True)
    file.touch()
    return {"created": path}


@router.delete("/workspace/projects/{project}/files/{path:path}")
async def delete_file(project: str, path: str):
    target = _file_path(project, path)
    if not target.exists():
        raise HTTPException(status_code=404, detail="Not found")
    if target.is_dir():
        shutil.rmtree(target)
    else:
        target.unlink()
    return {"deleted": path}


@router.post("/workspace/projects/{project}/rename")
async def rename_item(project: str, req: RenameRequest):
    src = _file_path(project, req.path)
    dst = _file_path(project, req.new_path)
    if not src.exists():
        raise HTTPException(status_code=404, detail="Source not found")
    if dst.exists():
        raise HTTPException(status_code=409, detail="Destination already exists")
    dst.parent.mkdir(parents=True, exist_ok=True)
    src.rename(dst)
    return {"renamed": req.path, "to": req.new_path}


@router.post("/workspace/projects/{project}/upload")
async def upload_file(project: str, file: UploadFile = File(...)):
    if not file.filename:
        raise HTTPException(status_code=400, detail="No filename")
    dest = _file_path(project, file.filename)
    dest.parent.mkdir(parents=True, exist_ok=True)
    with open(dest, "wb") as f:
        shutil.copyfileobj(file.file, f)
    return {"uploaded": file.filename, "size": dest.stat().st_size}


# ---- Project compilation ----

@router.post("/workspace/projects/{project}/compile")
async def compile_project(project: str, compiler: str = "pdflatex"):
    proj_path = _project_path(project)
    if not proj_path.exists():
        raise HTTPException(status_code=404, detail="Project not found")

    tex_files = sorted(proj_path.glob("*.tex"))
    if not tex_files:
        raise HTTPException(status_code=400, detail="No .tex files in project")

    main_file = tex_files[0].name

    patched = resolve_font_paths((proj_path / main_file).read_text(errors="replace"))
    (proj_path / main_file).write_text(patched)

    detected = detect_compiler(patched, compiler)
    success, log, warnings = run_compilation(proj_path, main_file, detected)

    pdf_path = proj_path / main_file.replace(".tex", ".pdf")
    pdf_exists = pdf_path.exists()

    return {
        "success": success,
        "pdf_exists": pdf_exists,
        "warnings": warnings,
        "compiler_used": detected,
    }


@router.get("/workspace/projects/{project}/pdf")
async def get_project_pdf(project: str):
    proj_path = _project_path(project)
    pdf_files = list(proj_path.glob("*.pdf"))
    if not pdf_files:
        raise HTTPException(status_code=404, detail="No PDF found. Compile the project first.")
    pdf = pdf_files[0]
    return Response(content=pdf.read_bytes(), media_type="application/pdf")


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
