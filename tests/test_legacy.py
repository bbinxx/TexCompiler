"""Characterization of v1 contracts; unsafe quirks are not v2 requirements."""
import io
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from fastapi.testclient import TestClient
from app import create_app
from app.compiler import detect_compiler, run_compilation

ROOT = Path(__file__).resolve().parents[1]

class LegacyContracts(unittest.TestCase):
    def setUp(self):
        self.client = TestClient(create_app())

    def test_empty_compile(self):
        response = self.client.post('/compile', json={})
        self.assertEqual(response.status_code, 400)
        self.assertEqual(response.json(), {'error': 'No code provided'})

    def test_detection(self):
        for code, preferred, expected in [
            ('plain', 'xelatex', 'xelatex'),
            (r'\usepackage{fontspec}', 'pdflatex', 'lualatex'),
            (r'\usepackage{xeCJK}', 'lualatex', 'xelatex'),
            (r'\usepackage{luacode}', 'xelatex', 'lualatex'),
            (r'\setmainfont{Example}', 'pdflatex', 'lualatex'),
        ]:
            with self.subTest(code=code):
                self.assertEqual(detect_compiler(code, preferred), expected)
                response = self.client.post('/detect-compiler', json={'code': code, 'preferred': preferred})
                self.assertEqual(response.status_code, 200)
                self.assertEqual(response.json(), {'compiler': expected})

    def test_health_and_compiler_inventory(self):
        response = self.client.get('/compilers')
        self.assertEqual(response.status_code, 200)
        engines = response.json()['compilers']
        self.assertEqual(engines, sorted(engines))
        health = self.client.get('/health')
        self.assertEqual(health.status_code, 200)
        self.assertEqual(health.json()['status'], 'healthy')
        self.assertEqual(health.json()['compilers'], engines)
        self.assertIn('version', health.json())
        self.assertIn('latexmk', health.json())

    def test_ide_and_static_client(self):
        self.assertEqual(self.client.get('/').status_code, 200)
        response = self.client.get('/static/js/compiler.js')
        self.assertEqual(response.status_code, 200)

    def test_bundle_errors(self):
        response = self.client.post('/compile/bundle', files={'file': ('main.tex', b'bad')})
        self.assertEqual(response.status_code, 400)
        self.assertEqual(response.json(), {'detail': 'Only ZIP bundles are supported.'})
        response = self.client.post('/compile/bundle', files={'file': ('bad.zip', b'bad')})
        self.assertEqual(response.status_code, 400)
        self.assertIn('error', response.json())
        data = io.BytesIO()
        with zipfile.ZipFile(data, 'w') as archive:
            archive.writestr('body.tex', 'body')
        response = self.client.post('/compile/bundle', files={'file': ('project.zip', data.getvalue())})
        self.assertEqual(response.status_code, 400)
        self.assertEqual(response.json(), {'error': "Main file 'main.tex' not found in bundle."})

    def test_missing_compiler_error(self):
        response = self.client.post('/compile', json={'code': 'plain', 'compiler': 'texcompiler-missing-test-engine'})
        self.assertEqual(response.status_code, 400)
        self.assertEqual(response.json()['error'], 'Compilation failed')
        self.assertIn('not found', response.json()['log'])
        self.assertEqual(response.json()['warnings'], [])

class RunnerCharacterization(unittest.TestCase):
    def test_nonzero_exit_with_pdf_is_currently_success(self):
        # Safe fake external process: reproduces the runner defect without TeX.
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            (workspace / 'main.pdf').write_bytes(b'not even a PDF')
            with patch('app.compiler.subprocess.run', return_value=subprocess.CompletedProcess([], 1, '', 'failure')):
                self.assertTrue(run_compilation(workspace, 'main.tex')[0])

    def test_timeout_error(self):
        import subprocess
        with tempfile.TemporaryDirectory() as directory:
            with patch('app.compiler.subprocess.run', side_effect=subprocess.TimeoutExpired('tex', 1)):
                success, log, warnings = run_compilation(Path(directory), 'main.tex')
        self.assertFalse(success)
        self.assertIn('timed out', log)
        self.assertEqual(warnings, [])

@unittest.skipUnless(os.environ.get('TEXCOMPILER_NATIVE_TESTS') == '1', 'opt-in trusted native fixtures')
class NativeCompatibility(unittest.TestCase):
    def setUp(self):
        self.client = TestClient(create_app())

    def check_pdf(self, response, text):
        self.assertEqual(response.status_code, 200, response.content[:2000])
        self.assertEqual(response.headers['content-type'], 'application/pdf')
        self.assertIn('attachment; filename=', response.headers['content-disposition'])
        self.assertTrue(response.content.startswith(b'%PDF-'))
        self.assertTrue(response.content.rstrip().endswith(b'%%EOF'))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'result.pdf'
            path.write_bytes(response.content)
            import subprocess
            info = subprocess.run(['pdfinfo', str(path)], capture_output=True, text=True, check=True)
            self.assertRegex(info.stdout, r'Pages:\s+1\b')
            extracted = subprocess.run(['pdftotext', str(path), '-'], capture_output=True, text=True, check=True)
            self.assertIn(text, extracted.stdout)

    def test_basic_each_engine(self):
        source = (ROOT / 'fixtures/basic/main.tex').read_text()
        for engine in ('pdflatex', 'xelatex', 'lualatex'):
            with self.subTest(engine=engine):
                self.assertIsNotNone(shutil.which(engine), engine)
                self.check_pdf(self.client.post('/compile', json={'code': source, 'compiler': engine}), 'TexCompiler compatibility baseline.')

    def test_multifile_and_zip(self):
        files = {p.name: p.read_text() for p in (ROOT / 'fixtures/multi-file').glob('*.tex')}
        self.check_pdf(self.client.post('/compile', json={'files': files, 'main_file': 'main.tex'}), 'Multi-file compatibility baseline.')
        data = io.BytesIO()
        with zipfile.ZipFile(data, 'w') as archive:
            for name, content in files.items():
                archive.writestr(name, content)
        self.check_pdf(self.client.post('/compile/bundle', files={'file': ('project.zip', data.getvalue())}), 'Multi-file compatibility baseline.')
