#!/usr/bin/env python3
import base64
import errno
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import tempfile
import time
from typing import IO, NotRequired, TypedDict


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


class LiveResult(RunResult):
    before: str
    waiting: bool
    retained: NotRequired[bool]


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


def read_startup_line(stream: IO[bytes]) -> bytes:
    data = b""
    deadline = time.monotonic() + 20
    while b"\n" not in data and time.monotonic() < deadline:
        if select.select([stream], [], [], 0.05)[0]:
            part = os.read(stream.fileno(), 8192)
            if not part:
                break
            data += part
    return data


def fifo_writer(fifo: Path) -> int:
    deadline = time.monotonic() + 10
    while True:
        try:
            return os.open(fifo, os.O_WRONLY | os.O_NONBLOCK)
        except OSError as error:
            if error.errno != errno.ENXIO or time.monotonic() >= deadline:
                raise
            time.sleep(0.05)


def fifo_startup(command: list[str], fifo: Path) -> LiveResult:
    process = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    assert process.stdout is not None
    try:
        before = read_startup_line(process.stdout)
        while select.select([process.stdout], [], [], 0.25)[0]:
            part = os.read(process.stdout.fileno(), 8192)
            if not part:
                break
            before += part
        waiting = process.poll() is None
        writer = fifo_writer(fifo)
        try:
            os.write(writer, b"cat('FIFO')\n")
        finally:
            os.close(writer)
        tail, stderr = process.communicate(timeout=20)
        assert process.returncode is not None
        return LiveResult(status=process.returncode,
                          before=base64.b64encode(before).decode("ascii"), waiting=waiting,
                          stdout=base64.b64encode(before + tail).decode("ascii"),
                          stderr=base64.b64encode(stderr).decode("ascii"))
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=10)


def retained_fifo(command: list[str], fifo: Path, source: bytes | None = None) -> LiveResult:
    process = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    assert process.stdout is not None
    writer = None
    try:
        writer = fifo_writer(fifo)
        if source is not None:
            os.write(writer, source)
            os.close(writer)
            writer = None
        before = read_startup_line(process.stdout)
        waiting = process.poll() is None
        try:
            if writer is None:
                writer = os.open(fifo, os.O_WRONLY | os.O_NONBLOCK)
            retained = os.write(writer, b"x") == 1
        except OSError as error:
            if error.errno not in [errno.ENXIO, errno.EPIPE]:
                raise
            retained = False
        process.kill()
        tail, stderr = process.communicate(timeout=10)
        assert process.returncode is not None
        return LiveResult(status=process.returncode, waiting=waiting, retained=retained,
                          before=base64.b64encode(before).decode("ascii"),
                          stdout=base64.b64encode(before + tail).decode("ascii"),
                          stderr=base64.b64encode(stderr).decode("ascii"))
    finally:
        if writer is not None:
            os.close(writer)
        if process.poll() is None:
            process.kill()
            process.wait(timeout=10)


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results: list[Comparison] = []
live_results: list[Comparison] = []
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
    second = root / "second.R"
    second.write_text("cat('SECOND')\n")
    invalid_source = root / "invalid-source.R"
    invalid_source.write_bytes(b"\xff")
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
    for label, args in [
        ("missing-file-with-expression", ["-f", str(missing), "-e", "1"]),
        ("late-warning-after-conflict", ["-e", "1", "-f", str(script), "-e", oversized]),
        ("warning-after-missing-file", ["-f", str(missing), "-e", oversized]),
    ]:
        cases.append((label, args, "exact", ""))
    for first in [script, missing, root, denied, invalid_source]:
        for last in [second, missing, root]:
            for tail in [[], ["-e", oversized], ["-e", "1"]]:
                cases.append((f"repeated-files-{first.name}-{last.name}-{len(tail)}",
                              ["-f", str(first), f"--file={last}", *tail], "exact", ""))
    for first in [script, missing, root]:
        for tail in [[], ["-e", oversized], ["-e", "1"]]:
            cases.append((f"stdin-after-file-{first.name}-{len(tail)}",
                          ["-f", str(first), "--file=-", *tail], "exact", ""))
    known = [
        ("escaped-tab-expression", ["-e", "\\t"], "Unquoted shell validation expands escapes after field splitting; native validation conflates these stages"),
        ("escaped-tab-filename", ["-f", "\\t"], "Empty filenames after shell field splitting are not implemented"),
        ("unknown-option", ["--bad-option"], "R warns and continues; native rejects unsupported options"),
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
        fifo = root / "startup.fifo"
        os.mkfifo(fifo)
        args = ["--vanilla", "--no-echo", "-e", oversized, "-f", str(fifo), "-e", "#" + "y" * 10000]
        r, u = [fifo_startup([command, *args], fifo) for command in [reference, str(snapshots[0])]]
        passed = r == u and r["waiting"] and r["status"] == 0 and not r["stderr"]
        passed = passed and base64.b64decode(r["before"]).startswith(b"WARNING: '-e #xxx")
        passed = passed and base64.b64decode(r["before"]).count(b"WARNING:") == 1
        passed = passed and base64.b64decode(r["stdout"]).endswith(b"FIFO")
        live_results.append(Comparison(label="warnings-around-blocking-file-open", args=args,
                                       kind="live", passed=passed, r=r, rust=u))
        holding = root / "holding.R"
        holding.write_text("cat('READY\\n');while(TRUE){}\n")
        args = ["--vanilla", "--no-echo", "-f", str(fifo), "-f", str(holding)]
        r, u = [retained_fifo([command, *args], fifo) for command in [reference, str(snapshots[0])]]
        passed = r == u and r["waiting"] and r["status"] == -9 and r.get("retained", False)
        passed = passed and base64.b64decode(r["before"]) == b"READY\n" and not r["stderr"]
        live_results.append(Comparison(label="earlier-file-stream-remains-open", args=args,
                                       kind="live", passed=passed, r=r, rust=u))
        for label, commands, args in [
            ("main-active-stream", [reference, str(snapshots[0])], ["--vanilla", "--no-echo", "-f", str(fifo)]),
            ("rscript-active-stream", ["Rscript", str(snapshots[1])], ["--vanilla", str(fifo)]),
        ]:
            r, u = [retained_fifo([command, *args], fifo, b"cat('READY\\n');while(TRUE){}\n")
                    for command in commands]
            passed = r == u and r["waiting"] and r["status"] == -9 and r.get("retained", False)
            passed = passed and base64.b64decode(r["before"]) == b"READY\n" and not r["stderr"]
            live_results.append(Comparison(label=label, args=args, kind="live", passed=passed, r=r, rust=u))
    finally:
        denied.chmod(0o600)

assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
Path("target/cli-startup-parity.json").write_text(json.dumps(dict(
    reference_version=version.stdout + version.stderr, source_sha256=sources,
    binary_sha256=hashes, cases=results, live_cases=live_results), indent=2) + "\n")
failed = [row for row in results if not row["passed"] and row["kind"] != "known"]
failed += [row for row in live_results if not row["passed"]]
known_results = [row for row in results if row["kind"] == "known"]
for row in failed:
    print(json.dumps(dict(label=row["label"], r_status=row["r"]["status"], rust_status=row["rust"]["status"])))
print(f"{sum(row['passed'] for row in results if row['kind'] != 'known')} passing startup comparisons; "
      f"{sum(row['passed'] for row in live_results)} live comparisons; "
      f"{len(failed)} unexpected differences; {sum(not row['passed'] for row in known_results)} known differences")
print("Help/version wording excluded; all other passing statuses and streams compare exactly.")
if failed:
    raise SystemExit(1)
