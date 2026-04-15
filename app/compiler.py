import os
import re
import shutil
import subprocess
from pathlib import Path

from app.config import LATEXMK_COMPATIBLE, COMPILE_TIMEOUT

KNOWN_TEX_COMPILERS = [
    "pdflatex", "xelatex", "lualatex", "latex", "pdftex", "xetex", "luatex",
    "luahbtex", "dvilualatex", "platex", "uplatex", "lualatex-dev", "luajittex",
    "luajitlatex", "dviluatex", "ps2pdf",
]

# Packages that require or strongly prefer lualatex
_LUALATEX_PKGS = r"\\usepackage(\[.*?\])?\{(luacode|luacolor|luamplib|luatexja|luabidi|luaplot)\}"
# Packages that require xelatex
_XELATEX_PKGS = r"\\usepackage(\[.*?\])?\{(xeCJK|xepersian)\}"
# Packages that work with xelatex or lualatex (auto-switch from pdflatex)
_UNICODE_PKGS = r"\\usepackage(\[.*?\])?\{fontspec\}"


_FONT_SEARCH_ROOTS = [
    "/usr/share/texmf-dist/fonts",
    *[f"/usr/local/texlive/{y}/texmf-dist/fonts" for y in range(2022, 2028)],
]
_FONT_CMD_RE = re.compile(
    r"(\\setmainfont|\\setsansfont|\\setmonofont)\{(\w+)\}\[([^\]]*)\]"
)


def _scan_font_dirs() -> list[Path]:
    dirs: list[Path] = []
    for root in _FONT_SEARCH_ROOTS:
        base = Path(root)
        if not base.is_dir():
            continue
        for sub in ("opentype", "truetype"):
            p = base / sub
            if p.is_dir():
                for vendor in sorted(p.iterdir()):
                    if vendor.is_dir():
                        dirs.append(vendor)
    return dirs


def _find_font(font_name: str, font_dirs: list[Path]) -> tuple[str | None, str | None]:
    search_dirs: list[Path] = []
    for d in font_dirs:
        search_dirs.append(d)
        search_dirs.extend(sorted(d.iterdir()))
    for d in search_dirs:
        if not d.is_dir():
            continue
        for ext in (".otf", ".ttf"):
            if (d / f"{font_name}-Regular{ext}").is_file():
                return str(d) + "/", ext
            if (d / f"{font_name}{ext}").is_file():
                return str(d) + "/", ext
            for f in d.iterdir():
                if f.name.startswith(font_name) and f.suffix == ext:
                    return str(d) + "/", ext
    return None, None


def resolve_font_paths(tex_code: str) -> str:
    if "\\setmainfont" not in tex_code and "\\setsansfont" not in tex_code and "\\setmonofont" not in tex_code:
        return tex_code

    font_dirs = _scan_font_dirs()

    def _replacer(m: re.Match) -> str:
        cmd = m.group(1)
        name = m.group(2)
        opts = m.group(3)

        path_m = re.search(r"Path\s*=\s*'?([^,'\]\s]+)", opts)
        ext_m = re.search(r"Extension\s*=\s*(\.?\w+)", opts)

        if not path_m:
            return m.group(0)

        spec_path = path_m.group(1)
        if os.path.isdir(spec_path):
            return m.group(0)

        found_dir, found_ext = _find_font(name, font_dirs)
        if found_dir:
            opts = opts[: path_m.start(1)] + found_dir + opts[path_m.end(1) :]
            if found_ext:
                ext_m = re.search(r"Extension\s*=\s*(\.?\w+)", opts)
                if ext_m:
                    opts = opts[: ext_m.start(1)] + found_ext + opts[ext_m.end(1) :]
            return f"{cmd}{{{name}}}[{opts}]"

        opts = re.sub(r",?\s*Path\s*=\s*'?[^,'\]\s]+'?,?\s*", "", opts)
        opts = re.sub(r",?\s*Extension\s*=\s*\.?\w+,?\s*", "", opts)
        opts = opts.strip().strip(",").strip()
        if opts:
            return f"{cmd}{{{name}}}[{opts}]"
        return f"{cmd}{{{name}}}"

    return _FONT_CMD_RE.sub(_replacer, tex_code)


def list_available_compilers() -> list[str]:
    return sorted(
        name for name in KNOWN_TEX_COMPILERS if shutil.which(name)
    )


def detect_compiler(code: str, preferred: str) -> str:
    if not preferred:
        preferred = "pdflatex"

    needs_lualatex = bool(re.search(_LUALATEX_PKGS, code))
    needs_xelatex = bool(re.search(_XELATEX_PKGS, code))
    needs_unicode = bool(re.search(_UNICODE_PKGS, code))

    if needs_lualatex:
        return "lualatex"
    if needs_xelatex:
        return "xelatex"
    if needs_unicode and preferred == "pdflatex":
        return "xelatex"

    return preferred


def build_compile_cmd(main_file: str, compiler: str) -> list[str]:
    if shutil.which("latexmk") and compiler in LATEXMK_COMPATIBLE:
        return [
            "latexmk",
            f"-{compiler}",
            "-pdf",
            "-interaction=nonstopmode",
            "-shell-escape",
            main_file,
        ]
    return [
        compiler,
        "-interaction=nonstopmode",
        "-shell-escape",
        main_file,
    ]


def extract_warnings(log_content: str) -> list[str]:
    if not log_content:
        return []
    warnings = re.findall(
        r"(?:LaTeX|Package|Class) Warning:.*?\n\n", log_content, re.DOTALL
    )
    if not warnings:
        warnings = [
            line.strip()
            for line in log_content.splitlines()
            if "Warning:" in line
        ]
    return warnings


def run_compilation(
    workdir: Path, main_file: str, compiler: str = "pdflatex"
) -> tuple[bool, str, list[str]]:
    cmd = build_compile_cmd(main_file, compiler)

    try:
        process = subprocess.run(
            cmd,
            cwd=workdir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=COMPILE_TIMEOUT,
        )
    except FileNotFoundError:
        return (
            False,
            f"Compiler '{compiler}' not found. Install texlive to proceed.",
            [],
        )
    except subprocess.TimeoutExpired:
        return (
            False,
            f"Compilation timed out after {COMPILE_TIMEOUT} seconds.",
            [],
        )
    except Exception as exc:
        return False, str(exc), []

    log_path = workdir / main_file.replace(".tex", ".log")
    log_content = ""
    if log_path.exists():
        log_content = log_path.read_text(errors="replace")

    warnings = extract_warnings(log_content)

    pdf_path = workdir / main_file.replace(".tex", ".pdf")
    pdf_exists = pdf_path.exists()

    if pdf_exists:
        return True, log_content, warnings

    return False, log_content or process.stdout or process.stderr, warnings
