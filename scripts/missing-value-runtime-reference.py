#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: uv run scripts/missing-value-runtime-reference.py OUTPUT_DIRECTORY
"""Capture direct top-level R missing-value boundaries without error handlers."""

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Final

CASES: Final = {
    "raw_lookup": "x <- quote(expr=); x",
    "global_missing": "x <- quote(expr=); print(missing(x))",
    "raw_forward": "x <- quote(expr=); print((function(y) missing(y))(x))",
    "named_hole_positional": "f <- function(x=99L) { print(missing(x)); print(x) }; f(x=, 7L)",
    "raw_function": "sum <- quote(expr=); sum(1)",
    "computed_function": "f <- function(sum) sum(1); f(quote(expr=))",
    "computed_repeat": (
        "n <- 0L; f <- function(x=99L) { print(missing(x)); print(typeof(x)); "
        "print(typeof(x)); print(missing(x)); print(n) }; f({ n <<- n+1L; quote(expr=) })"
    ),
    "default_repeat": (
        "n <- 0L; f <- function(x={ n <<- n+1L; quote(expr=) }) { print(missing(x)); "
        "print(typeof(x)); print(typeof(x)); print(missing(x)); print(n) }; f()"
    ),
    "computed_dots": (
        "n <- 0L; f <- function(...) { print(missing(..1)); print(typeof(..1)); "
        "print(typeof(..1)); print(missing(..1)); print(n) }; f({ n <<- n+1L; quote(expr=) })"
    ),
    "raw_dots": "f <- function(...) ..1; f(,)",
    "omitted_forward": "g <- function(y=99L) { print(missing(y)); print(typeof(y)) }; f <- function(x) g(x); f()",
    "computed_forward": (
        "g <- function(y=99L) { print(missing(y)); print(typeof(y)); print(missing(y)) }; "
        "f <- function(x) g(x); f(quote(expr=))"
    ),
    "explicit_default": "f <- function(x=99L) { print(missing(x)); print(x) }; f(x=)",
    "stored_computed": "f <- function(x) { y <- x; y }; f(quote(expr=))",
    "super_stored": "x <- 0L; f <- function(y) x <<- y; f(quote(expr=)); x",
    "cached_function": "f <- function(sum) { print(typeof(sum)); sum(1) }; f(quote(expr=))",
}


def main() -> None:
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=False)
    executable = os.environ.get("RSCRIPT", "Rscript")
    environment = {**os.environ, "R_ENABLE_JIT": "0", "LC_ALL": "C"}
    version = subprocess.run(
        [executable, "--vanilla", "-e", "cat(R.version.string)"],
        capture_output=True,
        check=True,
        env=environment,
    )
    if not version.stdout.startswith(b"R version 4.6.0 "):
        raise SystemExit("R 4.6.0 is required for this reference")
    _ = (output / "version.txt").write_bytes(version.stdout)
    rows: list[str] = []
    for name, source in CASES.items():
        path = output / (name + ".R")
        _ = path.write_text(source + "\n")
        runs: list[subprocess.CompletedProcess[bytes]] = []
        for suffix in ("", ".replay"):
            result = subprocess.run([executable, "--vanilla", str(path)], capture_output=True, env=environment)
            _ = (output / (name + suffix + ".stdout")).write_bytes(result.stdout)
            _ = (output / (name + suffix + ".stderr")).write_bytes(result.stderr)
            _ = (output / (name + suffix + ".exit")).write_text(str(result.returncode) + "\n")
            runs.append(result)
        first, replay = runs
        assert (replay.returncode, replay.stdout, replay.stderr) == (first.returncode, first.stdout, first.stderr), name
        rows.append(
            "\t".join([name, source, str(first.returncode), first.stdout.hex() or "-", first.stderr.hex() or "-"])
        )
    _ = (output / "reference.tsv").write_text("\n".join(rows) + "\n")
    manifest = {
        str(path.name): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(output.iterdir())
        if path.is_file()
    }
    _ = (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"{len(rows)} direct top-level cases and exact replays PASS")


if __name__ == "__main__":
    main()
