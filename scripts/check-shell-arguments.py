#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: uv run scripts/check-shell-arguments.py [path/to/r_ust_cli]
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

Kind = Literal["exact", "help", "version", "known", "guard"]


class Result(TypedDict):
    status: int
    stdout: str
    stderr: str


class Comparison(TypedDict):
    mode: str
    label: str
    args: list[str]
    kind: Kind
    passed: bool
    reference_run: bool
    r: Result | None
    rust: Result


def recorded(result: subprocess.CompletedProcess[bytes]) -> Result:
    return Result(status=result.returncode, stdout=base64.b64encode(result.stdout).decode("ascii"),
                  stderr=base64.b64encode(result.stderr).decode("ascii"))


def main() -> None:
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
    reference_r, reference_script = shutil.which("R"), shutil.which("Rscript")
    if not reference_r or not reference_script or Path(reference_r).resolve() == binary:
        raise SystemExit("Independent R/Rscript 4.6.0 references are required")
    environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL", LC_ALL="C")
    versions = [subprocess.run([program, "--version"], capture_output=True, text=True, check=True)
                for program in [reference_r, reference_script]]
    if any("version 4.6.0" not in result.stdout + result.stderr for result in versions):
        raise SystemExit("Reference R and Rscript must be version 4.6.0")
    sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
               for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                            Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
    rows: list[Comparison] = []
    with tempfile.TemporaryDirectory(prefix="r-ust shell arguments ") as directory:
        root = Path(directory)
        snapshots: list[Path] = []
        for original in [binary, binary.parent / "Rscript"]:
            snapshot = root / original.name
            _ = shutil.copy2(original, snapshot)
            snapshots.append(snapshot)
        hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
        file = root / "part file.R"
        _ = file.write_text("cat('FILE')\n")
        _ = (root / "marker\nfile.R").write_text("cat('MARKER')\n")
        _ = (root / "glob-a.R").write_text("cat('FIRST')\n")
        _ = (root / "glob-b.R").write_text("cat('SECOND')\n")
        for name in ["--bad-a", "--bad-b"]:
            _ = (root / name).write_text("")
        for is_main, reference, native, prefix in [
            (True, reference_r, str(snapshots[0]), ["--vanilla", "--no-echo"]),
            (False, reference_script, str(snapshots[1]), ["--vanilla"]),
        ]:
            cases: list[tuple[str, list[str], Kind]] = []
            for source in [r"\t", r"\x20", r"\040", r"\0-1", r"\t-1", r"\n-1",
                           r"\0\t", "\t", " \n\t", r"-x\n2", r"1\n\cignored",
                           r"1\n2\cignored", r"1\n\n\cignored"]:
                cases.append(("expression-validation", ["-e", source], "exact"))
            for source in [r"\t", r"\t-1", r"-n\n1", r"\n-1", r"\0-1"]:
                cases.append(("validation-before-effects", ["-e", "n<-2;cat('BEFORE')", "-e", source], "exact"))
            for source in [r"\t", "α β\n\t", r"\0-1", r"1\n\cignored"]:
                cases.append(("prepared-expression-arguments", ["-e", "writeLines(commandArgs()[-1])", "-e", source], "exact"))
            for word in ["--bad option", "--bad\toption", "--bad\noption", "--bad\u00a0option", r"--bad\toption"]:
                cases.append(("unquoted-fields", [word, "-e", "writeLines(commandArgs()[-1])"], "exact"))
            for suffix in [r"\tignored", r"\nignored", r"\cignored", "\tignored", "\nignored", r"\nkey=value", r"\n\n"]:
                raw = str(file) + suffix
                cases.append(("filename-fields", [*(["-f"] if is_main else []), raw], "exact"))
                if is_main:
                    cases.append(("inline-filename-fields", [f"--file={raw}"], "exact"))
            cases.append(("filename-markers", [*(["-f"] if is_main else []), str(root / "marker~n~file.R")], "exact"))
            trailing = ["", "two words", "\t", "\n", "~n~", "--help"]
            cases.append(("quoted-trailing-arguments", ["-e", "writeLines(commandArgs(TRUE))", *(["--args"] if is_main else []), *trailing], "exact"))
            for option in ["--max-nsize=10", "--max-vsize=" + "α" * 700]:
                cases.append(("removed-common-option", [option, "-e", "1"], "exact"))
                cases.append(("common-warning-before-version", [option, "--version", "-e", "1"], "version"))
                cases.append(("common-warning-before-native-warning", ["--bad", option, "-e", "cat('BODY')"], "exact"))
            for filename in ["x" * 1023, "x" * 1024, "x" * 1025, "x " * 256 + "x"]:
                cases.append(("filename-byte-limit", [*(["-f"] if is_main else []), filename], "exact"))
            if is_main:
                for option in ["-save", "-quiet", "-n", "-v", "-V"]:
                    cases.append(("removed-common-option", [option, "-e", "1"], "exact"))
                for args in [["ignored.R", "-e", "1"], ["version", "-e", "1"], ["--file", "ignored.R"], ["-"],
                             ["", "-e", r'cat("\\n")'], ["", "-e", ""],
                             ["--bad --help", "-e", "1"], ["-f", r"\t"]]:
                    cases.append(("native-argument-processing", args, "exact"))
                cases.extend([
                    ("common-warning-before-failed-open", ["--bad", "-f", "missing.R", "-quiet"], "exact"),
                    ("raw-help", ["ignored.R", "--help", "-e"], "help"),
                    ("generated-version", ["--bad --version", "-e", "1"], "version"),
                    ("filename-pathname-expansion", ["-f", "*.R"], "known"),
                ])
            cases.append(("option-pathname-expansion", ["--bad-*", "-e", "1"], "known"))
            for source in [r"1\cignored", r"cat('a\xffb')"]:
                cases.append(("unsafe-launcher-result", ["-e", source], "guard"))
            for label, args, kind in cases:
                u = subprocess.run([native, *prefix, *args], input=b"", env=environment, cwd=root,
                                   capture_output=True, timeout=20)
                r = None
                match kind:
                    case "guard":
                        passed = u.returncode == (2 if is_main else 1) and not u.stdout and u.stderr.startswith(b"-e launcher")
                    case "exact" | "help" | "version" | "known":
                        r = subprocess.run([reference, *prefix, *args], input=b"", env=environment, cwd=root,
                                           capture_output=True, timeout=20)
                        passed = (r.returncode, r.stdout) == (u.returncode, u.stdout)
                        if r.returncode == 1 and not r.stderr.startswith(b"ERROR:"):
                            passed = passed and bool(r.stderr) and bool(u.stderr) and not u.stderr.startswith(b"ERROR:")
                        else:
                            passed = passed and r.stderr == u.stderr
                        match kind:
                            case "help":
                                passed = r.returncode == u.returncode == 0 and not r.stderr and not u.stderr
                                passed = passed and r.stdout.lstrip().startswith(b"Usage:") and u.stdout.lstrip().startswith(b"Usage:")
                            case "version":
                                rp, rm, _ = r.stdout.partition(b"R version ")
                                up, um, _ = u.stdout.partition(b"R-ust ")
                                passed = r.returncode == u.returncode == 0 and not r.stderr and not u.stderr and bool(rm) and bool(um) and rp == up
                            case "exact" | "known":
                                pass
                            case _:
                                assert_never(kind)
                    case _:
                        assert_never(kind)
                rows.append(Comparison(mode="main" if is_main else "rscript", label=label, args=args,
                                       kind=kind, passed=passed, reference_run=r is not None,
                                       r=recorded(r) if r is not None else None, rust=recorded(u)))
            print(f"Checked {'main' if is_main else 'rscript'}: {len(cases)} cases", flush=True)
    assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
    _ = Path("target/shell-argument-parity.json").write_text(json.dumps(dict(
        versions=[r.stdout + r.stderr for r in versions], source_sha256=sources, binary_sha256=hashes,
        cases=rows), indent=2) + "\n")
    failed = [row for row in rows if not row["passed"] and row["kind"] != "known"]
    for row in failed:
        print(f"FAIL {row['mode']} {row['label']}: {row['args'][:2]!r}")
    print(f"{sum(row['passed'] for row in rows if row['kind'] not in ['known', 'guard'])} passing comparisons; {len(failed)} unexpected differences; {sum(row['kind'] == 'known' for row in rows)} pathname limits; {sum(row['kind'] == 'guard' for row in rows)} native guards without reference execution")
    print("Successful streams, launcher errors and statuses compare exactly. Runtime error and help/version wording are excluded; unsafe guards never run reference R.")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
