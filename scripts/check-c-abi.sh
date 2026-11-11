#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "$(uname -s)" == "Darwin" ]] && command -v sw_vers >/dev/null 2>&1; then
  export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-$(sw_vers -productVersion)}"
fi

cargo build -p r_ust_ffi

LIB="$ROOT/target/debug/libr_ust_ffi.a"
OUT="$ROOT/target/debug/c_abi_smoke"

cc tests/oracle/c_abi_smoke.c "$LIB" -lm -o "$OUT"
"$OUT"
