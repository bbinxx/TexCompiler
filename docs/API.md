# Legacy API baseline

Observed at commit 54c5d89; production is v1.3.0. The generated `/openapi.json`
is saved as `legacy-openapi.json`. It reflects actual routes but incompletely
describes binary/error responses. These observations supplement that snapshot.

| Route | Input | Observed output |
|---|---|---|
| POST /compile | JSON: optional code:string, files:map<string,string>, main_file=document.tex, compiler=pdflatex | 200 PDF + attachment filename; 400 error/log/warnings/details on engine failure |
| POST /compile/bundle | Multipart file, main_file=main.tex, compiler=pdflatex | 200 PDF; 400 detail for non-ZIP; error for extraction/missing main; error/log/warnings on compile failure |
| POST /detect-compiler | JSON code:string, preferred=pdflatex | 200 {compiler:string} |
| GET /compilers | none | 200 {compilers:sorted string array} |
| GET /health | none | status, version, compilers, latexmk path or Not found; can say healthy with no engines |
| GET / | none | Rendered IDE; /static serves browser assets |

Missing compile source returns 400 `{error: "No code provided"}`. PDF missing
after runner success returns 500. Framework validation uses 422 detail arrays.
Unhandled exceptions return 500 error/detail, currently including exception text.

`files` wins over `code` when writing assets. Detection uses code if nonempty,
otherwise concatenated file values. Binary extensions png/jpg/jpeg/pdf/cls/sty/zip
are base64-decoded; other values are text. In particular cls/sty are unexpectedly
treated as binary. ttf/otf are not binary extensions. Single-source requests
always write document.tex regardless of main_file and apply font rewriting.
ZIP requests use the supplied compiler directly without auto-detection.

Detection can override explicit preferences for Lua/Xe packages; fontspec and
font commands upgrade pdfTeX preferences to LuaLaTeX. Regexes can match comments
and miss comma-separated package lists. These quirks need explicit migration
policy; v2 must honor explicit engines and expose auto-selection separately.

The IDE posts only `{code,compiler}`, consumes raw PDF blobs, and reads
error/log/warnings on failure. It does not upload the full local workspace.
Vitae Forge and TexSpace were not checked externally; do not claim those clients
are verified from these server tests alone.

## Working Rust service (2.0.0-dev.3)

Run `./run-rust.sh`; the default listener is 127.0.0.1:8001. It implements the
five legacy endpoints above and embeds the original IDE/static assets. It uses
native isolated workers, with no Python interpreter in the request path.

- POST /compile and /api/v1/compile: legacy JSON input, binary PDF success.
- POST /api/v1/projects/compile: the same JSON multi-file contract.
- POST /compile/bundle: multipart ZIP, main_file=main.tex, optional compiler.
- POST /detect-compiler: code and preferred compiler -> compiler recommendation.
- GET /compilers and /api/v1/engines: installed allowlisted compilers.
- GET /health and /api/v1/health: runtime version, engines, startup-probed
  compilation_ready and production_ready=false.
- GET /api/v1/capabilities: native backend when startup probe succeeds,
  experimental=true and production_ready=false. No WASM capability is advertised.

Explicit engine selections are never overridden. Missing/blank compile engine
selections auto-detect Unicode/Lua/Xe package requirements; detection understands
comma-separated packages and ignores comments. The browser's blank loading
selection is accepted as auto. This intentionally corrects v1's silent overrides.

Binary assets (images/PDF/fonts/ZIP) are base64 strings in files. Plain TeX
class/package strings and the legacy base64 class/package convention are both
accepted. Source is not rewritten to substitute fonts or paths. ZIP input is
bounded and rejects unsafe paths, links/special entries and duplicate filenames.
The body limit is 32 MiB; decoded projects are limited to 20 MiB and 256 files.

Errors remain JSON: empty input is 400 No code provided; validation/engine/TeX
failures are 400; busy workers are 429; failed native setup is 503. Invalid JSON
uses the extractor's 4xx status with a JSON error. PDF success includes
Content-Disposition and X-TexCompiler-Engine, X-TexCompiler-Pages and
X-TexCompiler-Method headers. Attachment filenames use a sanitized main basename.

The old `router()` library function retains the noncompiling foundation router
for baseline tests; the executable uses `native_router()` and probes its worker.
The browser still submits its active source; the multi-file API is available
for clients but automatic upload of a local IDE workspace is not yet implemented.
Jobs, generated new OpenAPI, authentication, full package/font compatibility
and production deployment remain follow-ups. Do not expose this prototype to
untrusted users before its production gates pass.
