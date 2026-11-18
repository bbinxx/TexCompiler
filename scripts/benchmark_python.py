"""Trusted local baseline only; emits raw per-request timings, not speedup claims."""
import json
from pathlib import Path
import platform
import shutil
import subprocess
import time

from fastapi.testclient import TestClient
from app import create_app

root = Path(__file__).resolve().parents[1]
client = TestClient(create_app())
records = []
for engine in ('pdflatex', 'xelatex', 'lualatex'):
    version = subprocess.run([engine, '--version'], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    for attempt in range(4):
        started = time.perf_counter()
        response = client.post('/compile', json={'code': (root / 'fixtures/basic/main.tex').read_text(), 'compiler': engine})
        elapsed = time.perf_counter() - started
        if response.status_code != 200 or not response.content.startswith(b'%PDF-'):
            raise RuntimeError(f'{engine}: HTTP {response.status_code}')
        records.append({'engine': engine, 'version': version, 'attempt': attempt, 'seconds': elapsed, 'pdf_bytes': len(response.content)})
print(json.dumps({'python': platform.python_version(), 'platform': platform.platform(), 'fixture': 'fixtures/basic/main.tex', 'transport': 'in-process TestClient', 'latexmk': shutil.which('latexmk') is not None, 'records': records}, indent=2))
