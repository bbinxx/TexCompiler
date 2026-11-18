# Legacy Python reference

This is the retained FastAPI v1 reference, not the Rust v2 request path. It stays
only for compatibility comparison and the existing deployment until verified
Rust cutover; then this whole directory and its wrappers/CI will be removed.
The runtime reports 1.3.0; historical tags through v1.3.1 are preserved.

From the repository root, ./run.sh (or run.bat) delegates here. For explicit setup:

```sh
cd legacy/python
python -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-dev.txt
uvicorn main:app --host 127.0.0.1 --port 8000
```

Shared IDE files live under ../../web. The application resolves that path from
its location, so serving from this directory or via --app-dir works identically.
From the repository root, run reference tests and benchmarks with:

```sh
PYTHONPATH=legacy/python python -m unittest discover -s legacy/python/tests -v
PYTHONPATH=legacy/python TEXCOMPILER_NATIVE_TESTS=1 python -m unittest discover -s legacy/python/tests -v
PYTHONPATH=legacy/python python legacy/python/scripts/benchmark_python.py
```

| Setting | Default |
|---|---|
| PORT | 8000 |
| COMPILE_TIMEOUT | 120 seconds |
| LOG_LEVEL | INFO |
| CORS_ORIGINS | * |

The existing legacy compiler's security findings remain open; use trusted
fixtures only. The Rust worker isolation does not harden this old reference.
See ../../docs/SECURITY.md and ../../docs/MIGRATION.md.
