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

Versioned `/api/v1/*` routes, authentication, jobs, capabilities, and OpenAPI
for new features are planned in ARCHITECTURE.md; none exist yet.
