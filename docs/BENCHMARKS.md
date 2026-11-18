# Measurements and limitations

The migration records measurements instead of assuming Rust speeds up TeX.
Native TeX execution, package/font initialization, isolation and multipass work
dominate these examples; the API language alone does not establish improvement.

## Recorded observations

- python-baseline.json is the historical in-process reference timing snapshot.
- evidence/native-comparison.json records same-host HTTP compatibility and
  per-fixture single timings for Python and Rust. Seven fixtures match page
  count, extracted text, font names and 72 dpi page pixels.
- evidence/http-timings.json records four sequential observations per engine
  and implementation over loopback HTTP. The first observed request is separate
  from subsequent requests, but it is not a controlled OS-cache-cold experiment.

Both HTTP services use the same host TeX Live environment. Python runs one
unconfined pass; Rust runs three direct passes in private namespaces and does
not reuse project/font auxiliary caches. The Rust measurements are consequently
not equivalent work for a language-only comparison. No speedup is claimed.
Reproduce with a reference checkout when investigating a new optimization;
the main repository no longer requires Python.

## Future performance gate

Before claiming an optimization, benchmark the same engines/flags/package
snapshot, document corpus and isolation policy. Measure true cold and warm
behavior separately and report p50/p95, API/workspace/asset/engine/PDF phases,
memory/CPU, bounded-concurrency throughput and final image size. Include real
fonts, references, bibliography and large documents. Record failures, limits,
hardware and the exact source/image identifiers.

No controlled load/CPU/RSS benchmark, cache-hit comparison or incremental-build
result has been fabricated. Persistent caching, deduplication, queue scheduling
and optional WASM are follow-up work. Any cache must include all asset/font/
template bytes, engine/package environment, flags, relevant environment and
authorization boundaries, and must not reuse unrelated private intermediates.
