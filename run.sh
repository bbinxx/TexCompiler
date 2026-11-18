#!/usr/bin/env bash
# Compatibility launcher for the reference Python service.
set -euo pipefail
TC_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$TC_ROOT/legacy/python"
exec bash ./run.sh "$@"
