# Baseline measurements

Raw local samples: `python-baseline.json`, collected 2026-11-18 with
`legacy/python/scripts/benchmark_python.py`. Four sequential basic-document requests per
engine, unique workspace each time, in-process TestClient (includes route,
workspace, TeX execution and PDF read; excludes network and browser). First
request is first observed, not a controlled cold-cache measurement. Subsequent
requests can benefit from OS/font caches; there is no application result cache.
The three engine version strings and Python/platform are recorded in the JSON.

Native suite validates basic documents on pdfLaTeX/XeLaTeX/LuaLaTeX and JSON/ZIP
multi-file projects on pdfLaTeX, including extracted text and one-page output.
Local latexmk and Biber are absent; fallback direct compilation was exercised.
No Rust speedup is claimed. Samples are not enough for throughput or production
latency claims. CPU/RSS, phase-level timings, cold/warm cache control, HTTP
transport, concurrent load and bibliography timing remain unmeasured.

No TexCompiler image exists in the local Docker inventory. Container size is
unmeasured; no image build was performed. Docker currently installs a subset
of TeX Live (latex-extra, xetex, fonts-recommended, lang-all), latexmk and
Ghostscript on Python 3.11 slim. It does not explicitly install LuaTeX/Biber or
full TeX Live, and apt packages/base/dependencies are unpinned. A future runtime
must document minimal and maximum-compatibility profiles and measure both.

For a fair comparison, run both services on identical hardware, isolated worker
limits, engine/package/font snapshots and fixtures. Save per-request samples,
errors, page/text/font/visual validation, environment identity, and p50/p95
latency and throughput. Measure cold startup separately from steady state.
