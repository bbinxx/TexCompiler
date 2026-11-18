#!/usr/bin/env bash
set -euo pipefail
destination="${1:?Specify a license output directory}"
cargo_registry="${CARGO_HOME:-$HOME/.cargo}/registry/src"
test -d "$cargo_registry"
mkdir -p "$destination"
while IFS= read -r -d '' notice; do
  crate="$(basename -- "$(dirname -- "$notice")")"
  mkdir -p "$destination/$crate"
  cp -- "$notice" "$destination/$crate/"
done < <(find "$cargo_registry" -maxdepth 3 -type f \( -iname 'license*' -o -iname 'copying*' -o -iname 'notice*' -o -iname 'copyright*' \) -print0)
test -n "$(find "$destination" -type f -print -quit)"
