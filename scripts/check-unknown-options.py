#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: install uv with curl -LsSf https://astral.sh/uv/install.sh | sh
# Then run: uv run scripts/check-unknown-options.py [path/to/r_ust_cli]
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import Literal, TypedDict, assert_never


class Result(TypedDict):
    status: int
    stdout: str
    stderr: str


class Comparison(TypedDict):
    mode: str
    label: str
    args: list[str]
    kind: Literal["exact", "help", "version"]
    passed: bool
    r: Result
    rust: Result


def recorded(result: subprocess.CompletedProcess[bytes]) -> Result:
    return Result(status=result.returncode, stdout=base64.b64encode(result.stdout).decode("ascii"),
                  stderr=base64.b64encode(result.stderr).decode("ascii"))


def main() -> None:
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
    environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL", LC_ALL="C")
    reference_r, reference_script = shutil.which("R"), shutil.which("Rscript")
    if not reference_r or not reference_script or Path(reference_r).resolve() == binary:
        raise SystemExit("Independent R/Rscript 4.6.0 executables are required")
    versions = [subprocess.run([program, "--version"], capture_output=True, text=True, check=True)
                for program in [reference_r, reference_script]]
    if any("version 4.6.0" not in result.stdout + result.stderr for result in versions):
        raise SystemExit("Reference R and Rscript must be version 4.6.0")
    sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
               for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                            Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
    results: list[Comparison] = []
    with tempfile.TemporaryDirectory(prefix="r-ust-unknown-options-") as directory:
        root = Path(directory)
        snapshots: list[Path] = []
        for original in [binary, binary.parent / "Rscript"]:
            snapshot = root / original.name
            _ = shutil.copy2(original, snapshot)
            snapshots.append(snapshot)
        hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
        script, missing = root / "input.R", root / "missing.R"
        _ = script.write_text("cat('FILE')\n")
        oversized = "#" + "x" * 10000
        options = ["--bad-option", "-z", "--", "--bad-option=value", "--no-save=value",
                   "--vanilla=value", "--bad-option'α", "--σ",
                   *["--" + "x" * length + "α" * 8 for length in [1122, 1123, 1124]],
                   "--" + "α" * 700]
        for is_main, reference, native, prefix in [
            (True, reference_r, str(snapshots[0]), ["--vanilla", "--no-echo"]),
            (False, reference_script, str(snapshots[1]), ["--vanilla"]),
        ]:
            cases: list[tuple[str, list[str], Literal["exact", "help", "version"]]] = []
            for option in options:
                if not is_main and not option.startswith("--"):
                    continue
                cases.extend([
                    ("before-expression", [option, "-e", "cat('EXPR')"], "exact"),
                    ("repeated", [option, option, "-e", "cat('EXPR')"], "exact"),
                    ("before-file", [option, *(["-f"] if is_main else []), str(script)], "exact"),
                    ("before-missing-file", [option, *(["-f"] if is_main else []), str(missing)], "exact"),
                    ("argument-retention", [option, "-e", "writeLines(commandArgs()[-1])"], "exact"),
                    ("missing-expression", [option, "-e"], "exact"),
                ])
                if is_main:
                    cases.extend([
                        ("stdin-eof", [option], "exact"),
                        ("after-expression", ["-e", "cat('EXPR')", option], "exact"),
                        ("after-file", ["-f", str(script), option], "exact"),
                        ("after-missing-file", ["-f", str(missing), option], "exact"),
                        ("after-args", ["--args", option], "exact"),
                        ("help", [option, "--help"], "help"),
                        ("version", [option, "--version"], "version"),
                        ("conflict", [option, "-e", "1", "-f", str(script)], "exact"),
                        ("before-omission", [option, "-e", oversized], "exact"),
                        ("after-omission", ["-e", oversized, option], "exact"),
                    ])
                else:
                    cases.extend([
                        ("after-expression-is-argument", ["-e", "writeLines(commandArgs(TRUE))", option], "exact"),
                        ("after-file-is-argument", [str(script), option], "exact"),
                        ("missing-file-name", [option], "exact"),
                    ])
            if is_main:
                cases.extend(("shell-field-splitting", [option], "exact") for option in ["--bad option", "--bad\toption"])
            for label, args, kind in cases:
                r, u = [subprocess.run([program, *prefix, *args], input=b"", env=environment,
                                      capture_output=True, timeout=20) for program in [reference, native]]
                passed = (r.returncode, r.stdout) == (u.returncode, u.stdout)
                match kind:
                    case "help":
                        start = b"Usage:"
                    case "version":
                        start = b"R"
                    case "exact":
                        start = None
                    case _:
                        assert_never(kind)
                if start is not None:
                    passed = r.returncode == u.returncode == 0 and not r.stderr and not u.stderr
                    passed = passed and r.stdout.lstrip().startswith(start) and u.stdout.lstrip().startswith(start)
                else:
                    passed = passed and (r.stderr == u.stderr if is_main or r.returncode == 0 else bool(r.stderr) == bool(u.stderr))
                results.append(Comparison(mode="main" if is_main else "rscript", label=label, args=args,
                                          kind=kind, passed=passed, r=recorded(r), rust=recorded(u)))
            print(f"Checked {'main' if is_main else 'rscript'}: {len(cases)} comparisons", flush=True)
    assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
    _ = Path("target/unknown-option-parity.json").write_text(json.dumps(dict(
        versions=[result.stdout + result.stderr for result in versions], source_sha256=sources,
        binary_sha256=hashes, cases=results), indent=2) + "\n")
    failed = [row for row in results if not row["passed"]]
    for row in failed:
        print(f"FAIL {row['mode']}: {row['label']} {row['args'][0][:50]!r}")
    print(f"{sum(row['passed'] for row in results)} passing comparisons; {len(failed)} unexpected differences")
    print("Main streams/statuses and successful Rscript streams are exact; help/version and Rscript error wording are excluded.")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
