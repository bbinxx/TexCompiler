# Development

Production remains Python/FastAPI, version 1.3.0. Rust is planned, not shipped.

## Setup and verification

Use Python 3.11+ in a virtual environment. Install `requirements-dev.txt`.
Runtime dependencies remain unchanged; HTTPX is only a test dependency.

```sh
python -m pip install -r requirements-dev.txt
python -m unittest discover -s tests -v
TEXCOMPILER_NATIVE_TESTS=1 python -m unittest discover -s tests -v
PYTHONPATH=. python scripts/benchmark_python.py > docs/python-baseline.json
```

Native tests execute only repository-owned trusted fixtures and require
pdflatex, xelatex, lualatex, pdfinfo, and pdftotext. They validate PDF framing,
page count, extracted text, and both JSON and ZIP projects. The regular suite
uses no TeX execution except attempting a deliberately nonexistent executable.
The two mocked subprocess tests cover timeout translation and the v1 exit-code
defect; they do not validate sandboxing or process-tree termination.

The current host has TeX Live 2026/Arch Linux; latexmk and biber are absent.
Install latexmk and biber in the full integration environment. Package and
font snapshots are not pinned today. Do not run untrusted documents using v1.

## Workflow

Inspect status and diffs, fetch origin, fast-forward dev, create one focused
branch, implement tests before behavior, verify, review, and open a PR to dev.
Integrate only with green required checks, then fast-forward local dev and
remove the task branch/worktree after verified integration. Keep main and tags.
A clean audit worktree was created at `/tmp/texcompiler-phase0` on
`docs/migration-architecture`; original checkout stays on dev.

## Skills observed on 2026-11-18

Installed and read: writing-plans, using-git-worktrees,
test-driven-development (including writing-good-tests),
verification-before-completion. The full using-superpowers/brainstorming,
executing-plans/subagent-driven-development, finishing-a-development-branch,
Rust, and dedicated security-review/code-review packages were not found in
available catalogs/local skills. No unreviewed third-party skills were
installed. Use the available Superpowers-derived practices and explicit manual
security review. Review and install remaining guidance before Rust work.

New FastAPI/Starlette on this host emits a TestClient HTTPX deprecation warning;
the suite passes. Runtime requirements use minimum versions, not pins. A locked
Python test environment remains a follow-up reproducibility task.
