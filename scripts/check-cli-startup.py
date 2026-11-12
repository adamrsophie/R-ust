#!/usr/bin/env python3
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import NotRequired, TypedDict


class RunResult(TypedDict):
    status: int
    stdout: str
    stderr: str


class Comparison(TypedDict):
    label: str
    args: list[str]
    kind: str
    passed: bool
    reason: NotRequired[str]
    r: RunResult
    rust: RunResult


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
reference = shutil.which("R")
if not reference or Path(reference).resolve() == binary:
    raise SystemExit("An independent R 4.6.0 reference is required")
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL", LC_ALL="C")
version = subprocess.run([reference, "--version"], env=environment,
                         capture_output=True, text=True, check=True)
if "R version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("Reference must be R 4.6.0")


def recorded(result: subprocess.CompletedProcess[bytes]) -> RunResult:
    return RunResult(status=result.returncode,
                stdout=base64.b64encode(result.stdout).decode("ascii"),
                stderr=base64.b64encode(result.stderr).decode("ascii"))


results: list[Comparison] = []
with tempfile.TemporaryDirectory(prefix="r-ust-startup-") as directory:
    root = Path(directory)
    snapshots: list[Path] = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = root / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    script = root / "script with spaces.R"
    script.write_text("cat('FILE')\n")
    missing = root / "missing.R"
    dangling = root / "dangling.R"
    dangling.symlink_to(missing)
    denied = root / "denied.R"
    denied.write_text("cat('DENIED')\n")
    denied.chmod(0)
    non_directory = script / "child.R"
    cases: list[tuple[str, list[str], str, str]] = []
    for option in ["-e", "-f"]:
        for value in [None, "", " ", "\t", "\n", " \t\n", "-", "-1", "  -1", "--no-echo", "\\n"]:
            args = [option] if value is None else [option, value]
            cases.append((f"invalid-{option}-{value!r}", args, "exact", ""))
            cases.append((f"invalid-after-version-{option}-{value!r}", ["--version", *args], "exact", ""))
    for file in [script, missing, dangling, root, denied, non_directory]:
        for option in [["-f", str(file)], [f"--file={file}"]]:
            cases.append((f"file-{file.name}-{option[0]}", option, "exact", ""))
            cases.append((f"help-after-file-{file.name}", [*option, "--help"], "help", ""))
            cases.append((f"version-after-file-{file.name}", [*option, "--version"], "version", ""))
    cases.append(("empty-inline-file", ["--file="], "exact", ""))
    for file in [script, root]:
        for option in [["-f", str(file)], [f"--file={file}"]]:
            for args in [[*option, "-e", "1"], ["-e", "1", *option]]:
                cases.append(("file-expression-conflict", args, "exact", ""))
                cases.append(("launcher-error-before-conflict", [*args, "-e"], "exact", ""))
    for args in [["--help", "-e"], ["-e", "1", "--help", "-f"], ["--help", "--version"],
                 ["--version", "--help"]]:
        cases.append(("help-short-circuits", args, "help", ""))
    for args in [["-e", "-1", "--help"], ["--version", "-e"], ["-e", "1", "-f", "-"],
                 ["-f", str(missing), "-e"], ["-e", "1", "--version", "-f"]]:
        cases.append(("launcher-error-before-native-startup", args, "exact", ""))
    for args in [["--file=-", "-e", "cat('EXPR')"], ["-e", "cat('EXPR')", "--file=-"],
                 ["--file=-"], ["-e", "cat('EXPR')", "--args", "-f", "", "--help"]]:
        cases.append(("stdin-and-expression-selection", args, "exact", "cat('STDIN')\n"))
    oversized = "#" + "x" * 10000
    cases.append(("omitted-expression-before-conflict", ["-e", oversized, "-e", "1", "-f", str(script)], "exact", ""))
    cases.append(("raw-warning-before-conflict", ["-e", "#" + "α" * 5000, "-e", "1", "-f", str(script)], "exact", ""))
    known = [
        ("escaped-tab-expression", ["-e", "\\t"], "Unquoted shell validation expands escapes after field splitting; native validation conflates these stages"),
        ("escaped-tab-filename", ["-f", "\\t"], "Empty filenames after shell field splitting are not implemented"),
        ("unknown-option", ["--bad-option"], "R warns and continues; native rejects unsupported options"),
        ("missing-file-with-expression", ["-f", str(missing), "-e", "1"], "R opens files before checking the expression conflict"),
        ("late-warning-after-conflict", ["-e", "1", "-f", str(script), "-e", oversized], "R processes all admission warnings before checking the conflict"),
        ("warning-after-missing-file", ["-f", str(missing), "-e", oversized], "R stops at the file-open failure before later admission warnings"),
    ]
    cases.extend((label, args, "known", reason) for label, args, reason in known)
    try:
        for label, args, kind, data in cases:
            commands = [[reference, "--vanilla", "--no-echo", *args],
                        [str(snapshots[0]), "--vanilla", "--no-echo", *args]]
            r, u = [subprocess.run(command, input=(data if kind != "known" else "").encode(),
                                   env=environment, capture_output=True, timeout=20) for command in commands]
            equal = (r.returncode, r.stdout, r.stderr) == (u.returncode, u.stdout, u.stderr)
            passed = equal
            if kind in ["help", "version"]:
                prefix = b"Usage:" if kind == "help" else b"R"
                passed = r.returncode == u.returncode == 0 and not r.stderr and not u.stderr
                passed = passed and r.stdout.lstrip().startswith(prefix) and u.stdout.lstrip().startswith(prefix)
            results.append(Comparison(label=label, args=args, kind=kind, passed=passed,
                                reason=data if kind == "known" else "", r=recorded(r), rust=recorded(u)))
        for file in [script, missing, dangling, root, denied, non_directory]:
            args = ["--vanilla", str(file)]
            r, u = [subprocess.run([command, *args], env=environment, capture_output=True, timeout=20)
                    for command in ["Rscript", str(snapshots[1])]]
            results.append(Comparison(label=f"rscript-file-{file.name}", args=args, kind="exact",
                                passed=(r.returncode, r.stdout, r.stderr) == (u.returncode, u.stdout, u.stderr),
                                r=recorded(r), rust=recorded(u)))
    finally:
        denied.chmod(0o600)

sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
Path("target/cli-startup-parity.json").write_text(json.dumps(dict(
    reference_version=version.stdout + version.stderr, source_sha256=sources,
    binary_sha256=hashes, cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"] and row["kind"] != "known"]
known_results = [row for row in results if row["kind"] == "known"]
for row in failed:
    print(json.dumps(dict(label=row["label"], r_status=row["r"]["status"], rust_status=row["rust"]["status"])))
print(f"{sum(row['passed'] for row in results if row['kind'] != 'known')} passing startup comparisons; "
      f"{len(failed)} unexpected differences; {sum(not row['passed'] for row in known_results)} known differences")
print("Help/version wording excluded; all other passing statuses and streams compare exactly.")
if failed:
    raise SystemExit(1)
