import os
from pathlib import Path

APP_TITLE = "TexCompiler API"
APP_DESCRIPTION = "A high-performance LaTeX to PDF compilation service."
APP_VERSION = "1.1.0"  # single source of truth — change here, updates everywhere

LATEXMK_COMPATIBLE = frozenset({"pdflatex", "xelatex", "lualatex", "latex", "pdftex", "xetex", "luatex"})

COMPILE_TIMEOUT = int(os.environ.get("COMPILE_TIMEOUT", "120"))
LOG_LEVEL = os.environ.get("LOG_LEVEL", "INFO")
PORT = int(os.environ.get("PORT", "8000"))

CORS_ORIGINS = os.environ.get("CORS_ORIGINS", "*").split(",")

BINARY_EXTENSIONS = frozenset({
    ".png", ".jpg", ".jpeg", ".pdf", ".cls", ".sty", ".zip",
})

WORKSPACE_DIR = os.environ.get("WORKSPACE_DIR", str(Path(__file__).resolve().parent.parent / "workspace"))
