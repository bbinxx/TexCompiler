# API

The Rust service preserves the existing client shape: JSON source/projects and
multipart ZIP input, binary PDF on success, JSON diagnostics on failure. The
current OpenAPI contract is served at /openapi.json and stored in openapi.json.
legacy-openapi.json is historical baseline evidence, not the live contract.

## Routes

| Method | Legacy route | Versioned alias |
|---|---|---|
| POST | /compile | /api/v1/compile and /api/v1/projects/compile |
| POST | /compile/bundle | — |
| POST | /detect-compiler | — |
| GET | /compilers | /api/v1/engines |
| GET | /health | /api/v1/health |
| GET | /ready | — |
| GET | — | /api/v1/capabilities |

The root serves the IDE; /static URLs serve embedded web assets. Remote
listeners require a bearer token for compile/detect calls. Health, readiness,
inventory, capabilities, IDE/assets and OpenAPI remain public. A token can also
protect loopback use. Authentication is checked before project preparation.

## Source and projects

```json
{
  "compiler": "lualatex",
  "backend": "native",
  "main_file": "resume.tex",
  "files": {
    "resume.tex": "\\documentclass{article}\\begin{document}Hello\\end{document}",
    "body.tex": "Additional source"
  }
}
```

Alternatively submit {"code":"...","compiler":"pdflatex"}. A source-only
request writes document.tex, preserving the legacy behavior. main_file applies
to files projects, with default document.tex. A nonempty files map takes
precedence over code. No LaTeX source is modified to compensate for packages or
fonts. Project paths stay relative; ../, absolute/drive/backslash/control and
shell-metacharacter paths, option-like components and collisions are rejected.

Text values are UTF-8. Binary extensions png/jpg/jpeg/gif/webp/pdf/zip/ttf/otf/
woff/woff2 use standard base64. .cls/.sty accept ordinary TeX text or legacy
base64 values. ZIP transport preserves all uploaded bytes without extension-
based decoding. EPS/SVG assets may need installed converters and a supported
document workflow; shell escape is disabled.

compiler accepts pdflatex, xelatex or lualatex. Omit it or send an empty string
for conservative detection. Explicit choices are never silently overridden.
POST /detect-compiler accepts code and an optional preferred engine; it returns
{"compiler":"..."}. backend accepts native or auto; unsupported backends are
rejected rather than silently substituted. WASM is not implemented.

## ZIP input

```sh
curl --fail-with-body http://127.0.0.1:8001/compile/bundle \
  -H "Authorization: Bearer $TEXCOMPILER_API_TOKEN" \
  -F file=@project.zip -F main_file=main.tex -F compiler=xelatex \
  -o result.pdf
```

Use a .zip filename. main_file defaults to main.tex; omitted/blank compiler
auto-detects. ZIP members are validated before any filesystem writes. Duplicate
names, links, special files, traversal and oversized/decompression-bomb input
are rejected. Archives are not extracted using an unbounded system utility.

## Responses and limits

Success is application/pdf with Content-Disposition attachment filename plus
X-TexCompiler-Engine, X-TexCompiler-Pages and X-TexCompiler-Method. Nested project
main files produce a basename attachment. TeX's jobname keeps that basename.
The worker checks exit status and parses the generated PDF before returning it.

Compilation errors return 400 with error/log/warnings and optional details.
Validation errors return 400/422 JSON; authentication returns 401; oversized
bodies return 413; exhausted request/preparation/worker slots return 429;
unavailable native execution returns 503. A stalled whole request can return
408. Diagnostics are bounded and expose sandbox paths rather than host secrets.

| Limit | Value |
|---|---|
| HTTP request body | 32 MiB |
| Project files / ZIP entries | 256 |
| Decoded project / expanded archive | 20 MiB |
| Path / component bytes | 2048 / 255 |
| Concurrent admitted requests / preparations | 8 / 8 |
| Concurrent native workers | 2 per shared compiler instance |
| Compilation wall deadline | 120 seconds plus bounded cleanup |
| Whole-request deadline | 150 seconds |
| PDF / diagnostics | 16 MiB / 256 KiB |

No job/cache data persists between unrelated projects. The worker retains its
workspace and permit until process reaping even when the caller is cancelled.
latexmk performs multipass/bibliography work when installed; without it the
three-pass fallback does not guarantee bibliography automation.

## Health and capability semantics

/health is liveness and reports version, installed engines, latexmk and startup
compilation readiness. /ready returns 200 only after a real startup compilation,
otherwise 503. production_ready remains false pending stable-release acceptance.
Capabilities advertise the native backend only when the startup probe succeeds.
Asynchronous job APIs, persistent cache, SyncTeX output and WASM are not supported
and must not be inferred from planned v2 route names.

## Intentional legacy corrections

Explicit compiler selection, exit-code checking, disabled shell escape, strict
paths/ZIP limits and remote authentication replace unsafe/inconsistent v1 quirks.
Automatic host font-path rewriting and arbitrary executable names are removed.
These changes are documented rather than emulating vulnerabilities. See
CLIENTS.md for unchanged request models and authentication-header examples.

## Generate a PDF

With the local Rust service running on port 8001, save a PDF from a JSON request:

```sh
curl --fail-with-body -D pdf-headers.txt \
  -H 'Content-Type: application/json' \
  --data-binary '{"code":"\\documentclass{article}\\begin{document}Hello from TexCompiler\\end{document}","compiler":"pdflatex"}' \
  http://127.0.0.1:8001/compile -o hello.pdf
```

Check that curl exits successfully before opening the file. On failure the file
can contain JSON diagnostics instead of a PDF. For a token-protected server,
add `-H "Authorization: Bearer $TEXCOMPILER_API_TOKEN"` to each POST example.

## Build a multi-file project

```sh
curl --fail-with-body http://127.0.0.1:8001/api/v1/projects/compile \
  -H 'Content-Type: application/json' \
  --data-binary '{"main_file":"main.tex","files":{"main.tex":"\\documentclass{article}\\begin{document}\\input{body}\\end{document}","body.tex":"Text from a second file."}}' \
  -o project.pdf
```

For a project with binary assets, use the ZIP example above or encode each font
or image as base64 in the JSON `files` map. Keep the original paths. For example,
place an uploaded font at `fonts/Regular.otf`, select `xelatex` or `lualatex`,
and use `\setmainfont[Path=fonts/]{Regular.otf}` with `fontspec`.
Include bibliography databases and style files in the project. Install latexmk
and the required BibTeX or Biber executable on the server for bibliography builds.

## Select an engine and inspect the service

```sh
curl --fail http://127.0.0.1:8001/compilers
curl --fail http://127.0.0.1:8001/health
curl --fail http://127.0.0.1:8001/ready
curl --fail http://127.0.0.1:8001/api/v1/capabilities
curl --fail http://127.0.0.1:8001/detect-compiler \
  -H 'Content-Type: application/json' \
  --data-binary '{"code":"\\usepackage{fontspec}","preferred":"pdflatex"}'
```

Detection returns a recommendation and does not compile. Use the returned engine
in a subsequent request or omit `compiler` and let the service detect it. An
explicit engine is used as provided. The `X-TexCompiler-Engine`,
`X-TexCompiler-Pages` and `X-TexCompiler-Method` headers describe the completed
build. Check the status before interpreting the body and see the response table
above for retry and error handling. For integration snippets, see [Client integration](CLIENTS.md).
