# Client integration

The service accepts TeX and associated assets; it contains no resume/recipe
business logic. Use the same protocol from Vitae Forge, TexSpace or another
client. JSON models need not change: add an Authorization header when connecting
to an authenticated deployment. Successful compile responses are binary PDFs,
not JSON/base64. Parse JSON only after a non-success HTTP status.

## Server-side JavaScript

```js
const response = await fetch(`${process.env.TEXCOMPILER_URL}/compile`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${process.env.TEXCOMPILER_API_TOKEN}`,
  },
  body: JSON.stringify({
    main_file: 'resume.tex',
    compiler: 'lualatex',
    files: {
      'resume.tex': generatedLatex,
      'fonts/Regular.otf': fontBuffer.toString('base64'),
      'photo.png': imageBuffer.toString('base64'),
    },
  }),
});
if (!response.ok) throw new Error(JSON.stringify(await response.json()));
const pdf = Buffer.from(await response.arrayBuffer());
```

Keep service credentials on the server or a trusted gateway. Browser users can
enter a token into the built-in IDE's tab-scoped Settings. Do not bundle a shared
service token into a public React application. Vitae Forge continues generating
its own LaTeX; this repository does not migrate that generator, Firestore or R2.

## Kotlin / Ktor

```kotlin
@Serializable
data class ApiCompileRequest(
    val main_file: String,
    val compiler: String = "pdflatex",
    val files: Map<String, String>,
)

val response = client.post("$baseUrl/compile") {
    contentType(ContentType.Application.Json)
    bearerAuth(apiToken)
    setBody(ApiCompileRequest("main.tex", "pdflatex", projectFiles))
}
if (response.status.isSuccess()) {
    val pdf: ByteArray = response.readRawBytes()
    // Store or display PDF bytes in the platform client.
} else {
    val diagnostics: String = response.bodyAsText()
    // Present the JSON error/log without interpreting it as a PDF.
}
```

Use the appropriate Ktor engine and kotlinx.serialization content-negotiation
plugin for your client. The local TexSpace ApiCompileRequest fields and binary
response handling match this contract. Its hardcoded pdflatex remains explicit:
select a Unicode engine for fontspec templates rather than expecting the server
to override it. Supply uploaded binary assets as base64 and keep main_file
relative to the project root.

## Detection and errors

Omit compiler or pass an empty string for conservative automatic detection, or
call POST /detect-compiler with code and an optional preferred engine. Explicit
engines are always honored. backend accepts native or auto; wasm is rejected.
Retry 429/503 only with bounded backoff; do not blindly retry syntax errors.
Cancellation of the in-process request reaps its sandbox before releasing the
workspace/slot. Persistent job APIs are not implemented.

## Evidence and limits

Native HTTP fixtures, JSON/ZIP projects, uploaded fonts, errors and the rendered
IDE flow are tested. Seven legacy-reference fixtures match pages, text, font
names and rendered pixels. TexSpace's local request/response schema was read;
no live Android/iOS/web client build or external account was exercised. Client-
owned Vitae Forge templates were not available locally. Validate those actual
templates and live deployment separately before claiming end-to-end acceptance.
