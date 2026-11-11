#!/usr/bin/env bash
# Rebuild documentation/PORTING.md. Requires R 4.6.0 (Rscript) and python3.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
D="$ROOT/documentation/porting-data"
cd "$ROOT"
cargo build -q --release -p r_ust_cli
BIN="$ROOT/target/release/r_ust_cli"
Rscript --vanilla "$D/r_inventory.R" "$D/r_inventory.tsv"
python3 -I "$D/probe.py" "$D/r_inventory.tsv" "$BIN" "$D/probe.tsv"
python3 -I "$D/formals.py" crates/r_ust_eval/src/lib.rs "$D/r_inventory.tsv" "$D/probe.tsv" "$D/formals.tsv"
python3 -I "$D/run_all.py" "$D" "$BIN"
python3 -I "$D/check_operators.py" "$D" "$BIN"
python3 -I "$D/check_missing.py" "$D" "$BIN"
python3 -I "$D/check_integers.py" "$D" "$BIN"
REVISION="$(git rev-parse --short HEAD)"
if [[ -n "$(git status --porcelain)" ]]; then
    REVISION="$REVISION-dirty"
fi
python3 -I "$D/make_porting.py" "$D" documentation/PORTING.md "$REVISION" crates/r_ust_nmath/src/surface.rs
