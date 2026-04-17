#!/usr/bin/env bash
set -e

# ------------------------------------------------------------------
#  TexCompiler — Cross-platform launcher (Linux / macOS / Git Bash)
# ------------------------------------------------------------------

VENV_DIR=".venv"
REQUIREMENTS="requirements.txt"
APP_ENTRY="main.py"
PORT="${PORT:-8000}"

# ---- detect OS / shell environment ----
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    OS="windows"
    PYTHON_CMD="${PYTHON_CMD:-python}"
    VENV_ACTIVATE="$VENV_DIR/Scripts/activate"
    ;;
  Darwin)
    OS="macos"
    PYTHON_CMD="${PYTHON_CMD:-python3}"
    VENV_ACTIVATE="$VENV_DIR/bin/activate"
    ;;
  Linux|*)
    OS="linux"
    PYTHON_CMD="${PYTHON_CMD:-python3}"
    VENV_ACTIVATE="$VENV_DIR/bin/activate"
    ;;
esac

echo "==> Detected OS: $OS"

# ---- find Python ----
if ! command -v "$PYTHON_CMD" &>/dev/null; then
  # fallback: try the other common name
  if [ "$PYTHON_CMD" = "python3" ]; then
    PYTHON_CMD="python"
  else
    PYTHON_CMD="python3"
  fi
  if ! command -v "$PYTHON_CMD" &>/dev/null; then
    echo "Error: Python is not installed. Install Python 3.10+ first."
    echo "  - Debian/Ubuntu: sudo apt install python3 python3-venv"
    echo "  - Fedora:        sudo dnf install python3"
    echo "  - Arch:          sudo pacman -S python"
    echo "  - macOS:         brew install python"
    echo "  - Windows:       https://python.org/downloads"
    exit 1
  fi
fi

echo "==> Using Python: $PYTHON_CMD ($("$PYTHON_CMD" --version 2>&1))"

# ---- virtual environment ----
if [ ! -d "$VENV_DIR" ]; then
  echo "==> Creating virtual environment..."
  "$PYTHON_CMD" -m venv "$VENV_DIR"
fi

echo "==> Activating virtual environment..."
# shellcheck source=/dev/null
. "$VENV_ACTIVATE"

# ---- upgrade pip & install deps ----
pip install --upgrade pip -q
if [ -f "$REQUIREMENTS" ]; then
  pip install -r "$REQUIREMENTS" -q
fi

# ---- check LaTeX availability ----
LATEX_OK=true
command -v pdflatex &>/dev/null || { echo "Warning: pdflatex not found."; LATEX_OK=false; }
command -v xelatex  &>/dev/null || { echo "Warning: xelatex not found.";  LATEX_OK=false; }
command -v lualatex &>/dev/null || { echo "Warning: lualatex not found."; LATEX_OK=false; }

if [ "$LATEX_OK" = false ]; then
  echo ""
  echo "  A LaTeX distribution is required for compilation."
  echo "  Install one of:"
  echo "    - TeX Live (Linux/macOS/WSL): https://tug.org/texlive/"
  echo "    - MiKTeX   (Windows):         https://miktex.org/"
  echo "    - BasicTeX (macOS):           brew install basictex"
  echo ""
fi

# ---- free up port if in use ----
if command -v fuser &>/dev/null; then
  fuser -k "${PORT}/tcp" 2>/dev/null && echo "==> Freed port $PORT" || true
fi

# ---- start service ----
echo "==> Starting TexCompiler on http://localhost:$PORT"
echo ""
uvicorn main:app --host 0.0.0.0 --port "$PORT" --reload
