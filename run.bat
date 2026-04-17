@echo off
setlocal enabledelayedexpansion

REM ------------------------------------------------------------------
REM  TexCompiler — Windows launcher (cmd.exe)
REM ------------------------------------------------------------------

set VENV_DIR=.venv
set REQUIREMENTS=requirements.txt
set APP_ENTRY=main.py

if "%PORT%"=="" set PORT=8000

echo ==^> Detected OS: Windows

REM ---- find Python ----
where python >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: Python is not installed. Install Python 3.10+ from https://python.org/downloads/
    pause
    exit /b 1
)

python --version 2>nul | find "3." >nul
if %ERRORLEVEL% neq 0 (
    echo Error: Python 3.x is required.
    pause
    exit /b 1
)

echo ==^> Using Python: %PYTHON_CMD% 
python --version

REM ---- virtual environment ----
if not exist "%VENV_DIR%" (
    echo ==^> Creating virtual environment...
    python -m venv "%VENV_DIR%"
)

echo ==^> Activating virtual environment...
call "%VENV_DIR%\Scripts\activate.bat"

REM ---- upgrade pip ^& install deps ----
python -m pip install --upgrade pip -q
if exist "%REQUIREMENTS%" (
    pip install -r "%REQUIREMENTS%" -q
)

REM ---- check LaTeX availability ----
where pdflatex >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Warning: pdflatex not found.
    echo.
    echo   A LaTeX distribution is required for compilation.
    echo   Install MiKTeX from: https://miktex.org/
    echo.
)

REM ---- free up port if in use ----
for /f "tokens=5" %%a in ('netstat -ano ^| find ":%PORT%" ^| find "LISTENING"') do (
    echo ==^> Killing process on port %PORT% (PID: %%a)
    taskkill /F /PID %%a >nul 2>nul
)

REM ---- start service ----
echo ==^> Starting TexCompiler on http://localhost:%PORT%
echo.
uvicorn main:app --host 0.0.0.0 --port %PORT% --reload

pause
endlocal
