#!/usr/bin/env bash
set -euo pipefail
TC_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$TC_ROOT"
cargo build --locked --bins --target-dir "$TC_ROOT/target"
exec env TEXCOMPILER_WORKER="${TEXCOMPILER_WORKER:-$TC_ROOT/target/debug/texcompiler-worker}" "$TC_ROOT/target/debug/texcompiler"
