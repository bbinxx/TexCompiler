#!/usr/bin/env bash
set -euo pipefail
TC_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$TC_ROOT/scripts/dev/run-rust.sh" "$@"
