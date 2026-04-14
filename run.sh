#!/bin/bash

# Configuration
VENV_DIR=".venv"
REQUIREMENTS="requirements.txt"
APP_ENTRY="main.py"

echo "🚀 Starting TexCompiler Setup..."

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: python3 is not installed."
    exit 1
fi

# Create virtual environment if it doesn't exist
if [ ! -d "$VENV_DIR" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
    if [ $? -ne 0 ]; then
        echo "❌ Error: Failed to create virtual environment. You may need to install python3-venv (sudo apt install python3-venv)."
        exit 1
    fi
fi

# Activate virtual environment
echo "🔌 Activating environment..."
source "$VENV_DIR/bin/activate"

# Install/Update dependencies
if [ -f "$REQUIREMENTS" ]; then
    echo "📥 Installing dependencies from $REQUIREMENTS..."
    pip install --upgrade pip
    pip install -r "$REQUIREMENTS"
else
    echo "⚠️ Warning: $REQUIREMENTS not found. Skipping dependency installation."
fi

# Check for LaTeX dependencies
if ! command -v pdflatex &> /dev/null; then
    echo "⚠️ Warning: pdflatex not found. Local compilation will fail."
    echo "💡 Suggestion: install texlive (sudo apt install texlive-full)"
fi

if ! command -v latexmk &> /dev/null; then
    echo "⚠️ Warning: latexmk not found. Using raw pdflatex might cause issues with complex docs."
fi

echo "✨ Starting TexCompiler Service..."
echo "📊 Accessible at: http://localhost:8000"
echo "------------------------------------------------"

# Run the application
python3 "$APP_ENTRY"
