from typing import Optional

from pydantic import BaseModel


class CompileRequest(BaseModel):
    code: Optional[str] = None
    files: Optional[dict[str, str]] = None
    main_file: str = "document.tex"
    compiler: str = "pdflatex"


class DetectCompilerRequest(BaseModel):
    code: str
    preferred: str = "pdflatex"


class CompileErrorResponse(BaseModel):
    error: str
    log: Optional[str] = None
    warnings: list[str] = []
    details: Optional[str] = None


class CompileBundleForm:
    def __init__(self, main_file: str = "main.tex", compiler: str = "pdflatex"):
        self.main_file = main_file
        self.compiler = compiler


class HealthResponse(BaseModel):
    status: str
    pdflatex: str
    latexmk: str


class CreateProjectRequest(BaseModel):
    name: str


class CreateFileRequest(BaseModel):
    path: str
    content: str = ""


class CreateFolderRequest(BaseModel):
    path: str


class RenameRequest(BaseModel):
    path: str
    new_path: str


class ProjectFile(BaseModel):
    name: str
    path: str
    type: str  # "file" or "dir"
    size: int = 0
