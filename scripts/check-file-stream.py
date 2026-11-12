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
from typing import TypedDict


class Case(TypedDict):
    label: str
    phases: list[tuple[str, str, str | None]]
    status: int
    terminal: bool


class Snapshot(TypedDict):
    stdout: str
    stderr: str
    running: bool
    matched: bool


class Result(TypedDict):
    status: int
    stdout: str
    stderr: str


class StreamResult(Result):
    phases: list[Snapshot]
    exited_before_eof: bool


class Comparison(TypedDict):
    label: str
    mode: str
    kind: str
    passed: bool
    r: Result
    rust: Result


def encoded(data: bytes) -> str:
    return base64.b64encode(data).decode("ascii")


def recorded(result: subprocess.CompletedProcess[bytes]) -> Result:
    return Result(status=result.returncode, stdout=encoded(result.stdout), stderr=encoded(result.stderr))


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R/Rscript 4.6.0 installation is required")
reference_r = shutil.which("R")
if not reference_r or Path(reference_r).resolve() == binary:
    raise SystemExit("An independent R 4.6.0 executable is required")
r_version = subprocess.run([reference_r, "--version"], capture_output=True, text=True, check=True)
if "R version 4.6.0" not in r_version.stdout + r_version.stderr:
    raise SystemExit("Reference R must be version 4.6.0")
locale = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True).stdout


def streamed(command: list[str], fifo: Path, case: Case) -> StreamResult:
    process = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0)
    assert process.stdout is not None and process.stderr is not None
    output, errors = bytearray(), bytearray()
    active = {process.stdout.fileno(): output, process.stderr.fileno(): errors}

    def drain(wait: float) -> None:
        ready, _, _ = select.select(list(active), [], [], wait)
        for descriptor in ready:
            part = os.read(descriptor, 65536)
            if part:
                active[descriptor].extend(part)
            else:
                del active[descriptor]

    writer = None
    states: list[Snapshot] = []
    try:
        deadline = time.monotonic() + 10
        while True:
            try:
                writer = os.open(fifo, os.O_WRONLY | os.O_NONBLOCK)
                break
            except OSError as error:
                if error.errno != errno.ENXIO or time.monotonic() >= deadline:
                    raise
                time.sleep(0.05)
        for source, expected, expected_error in case["phases"]:
            data = source.encode()
            try:
                assert os.write(writer, data) == len(data)
            except BrokenPipeError:
                break
            deadline = time.monotonic() + 10
            while len(output) < len(expected.encode()) and process.poll() is None and time.monotonic() < deadline:
                drain(0.05)
            drain(0.1)
            matched = bytes(output) == expected.encode()
            matched = matched and (expected_error is None or bytes(errors) == expected_error.encode())
            states.append(Snapshot(stdout=encoded(bytes(output)), stderr=encoded(bytes(errors)),
                                   running=process.poll() is None, matched=matched))
        if case["terminal"]:
            deadline = time.monotonic() + 10
            while process.poll() is None and time.monotonic() < deadline:
                drain(0.05)
        exited = process.poll() is not None
        os.close(writer)
        writer = None
        tail, stderr = process.communicate(timeout=20)
        assert process.returncode is not None
        return StreamResult(status=process.returncode, stdout=encoded(bytes(output) + tail),
                            stderr=encoded(bytes(errors) + stderr), phases=states, exited_before_eof=exited)
    finally:
        if writer is not None:
            os.close(writer)
        if process.poll() is None:
            process.kill()
            process.wait(timeout=10)


cases = [
    Case(label="lines", phases=[("cat('READY\\n')\n", "READY\n", ""),
                                ("cat('TAIL\\n')\n", "READY\nTAIL\n", "")], status=0, terminal=False),
    Case(label="continuation", phases=[("cat('READY\\n')\nsum(\n", "READY\n", ""),
                                       ("1,2)\ncat('TAIL\\n')\n", "READY\n[1] 3\nTAIL\n", "")], status=0, terminal=False),
    Case(label="partial-line", phases=[("cat('READY\\n')\ncat('TA", "READY\n", ""),
                                       ("IL\\n')\n", "READY\nTAIL\n", "")], status=0, terminal=False),
    Case(label="split-crlf", phases=[("cat('READY\\n')\r\n1+\r", "READY\n", ""),
                                     ("\n2\r\ncat('TAIL\\n')\r\n", "READY\n[1] 3\nTAIL\n", "")], status=0, terminal=False),
    Case(label="literal-warning", phases=[("options(warn=1)\ncat('READY\\n')\n1.5L+\n", "READY\n", ""),
                                          ("2\ncat('TAIL\\n')\n", "READY\n[1] 3.5\nTAIL\n", None)], status=0, terminal=False),
    Case(label="echo", phases=[("options(echo=TRUE)\nsum(\n", "> sum(\n+ ", ""),
                               ("1,2)\n", "> sum(\n+ 1,2)\n[1] 3\n> ", "")], status=0, terminal=False),
    Case(label="quit", phases=[("cat('READY\\n');q('no',status=3)\n", "READY\n", "")], status=3, terminal=True),
    Case(label="parse-error", phases=[("cat('READY\\n')\n@\n", "READY\n", None)], status=1, terminal=True),
    Case(label="eval-error", phases=[("cat('READY\\n')\nmissingName\n", "READY\n", None)], status=1, terminal=True),
    Case(label="warning-error", phases=[("options(warn=2)\ncat('READY\\n')\n1.5L\n", "READY\n", None)], status=1, terminal=True),
    Case(label="incomplete-eof", phases=[("cat('READY\\n')\n1+\n", "READY\n", "")], status=1, terminal=False),
]
byte_cases: list[tuple[str, bytes, int, str]] = [
    ("quit-before-invalid-bytes", b"cat('BEFORE')\nq('no')\n\x80\n", 0, "bytes"),
    ("error-after-output", b"cat('BEFORE')\n\x80\n", 1, "bytes"),
    ("same-line-byte-validation", b"cat('BEFORE')\nq('no');\x80\n", 0, "bytes"),
    ("truncated-multibyte-eof", b"cat('BEFORE')\n\xff\n", 0, "known"),
]
sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results: list[Comparison] = []
with tempfile.TemporaryDirectory(prefix="r-ust-file-stream-") as directory:
    root = Path(directory)
    snapshots: list[Path] = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = root / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    fifo = root / "input.fifo"
    os.mkfifo(fifo)
    for mode, reference, native in [
        ("main", [reference_r, "--vanilla", "--no-echo", "-f"], [str(snapshots[0]), "--vanilla", "--no-echo", "-f"]),
        ("rscript", ["Rscript", "--vanilla"], [str(snapshots[1]), "--vanilla"]),
    ]:
        for case in cases:
            r, u = [streamed([*command, str(fifo)], fifo, case) for command in [reference, native]]
            passed = r["status"] == u["status"] == case["status"] and r["stdout"] == u["stdout"]
            passed = passed and (r["stderr"] == u["stderr"] if case["status"] in [0, 3] else bool(r["stderr"]) and bool(u["stderr"]))
            for run in [r, u]:
                passed = passed and len(run["phases"]) == len(case["phases"]) and all(state["matched"] for state in run["phases"])
                passed = passed and (run["exited_before_eof"] if case["terminal"] else all(state["running"] for state in run["phases"]))
            if case["label"] == "warning-error":
                passed = passed and all(b"(converted from warning)" in base64.b64decode(run["stderr"]) for run in [r, u])
            results.append(Comparison(label=case["label"], mode=mode, kind="live", passed=passed, r=r, rust=u))
        print(f"Checked {mode}: {len(cases)} live file scenarios", flush=True)
        path = root / "bytes.R"
        for label, source, status, kind in byte_cases:
            path.write_bytes(source)
            r, u = [subprocess.run([*command, str(path)], env=environment, capture_output=True, timeout=20)
                    for command in [reference, native]]
            passed = r.returncode == u.returncode == status and r.stdout == u.stdout == b"BEFORE"
            passed = passed and (not r.stderr and not u.stderr if status == 0 else bool(r.stderr) and bool(u.stderr))
            results.append(Comparison(label=label, mode=mode, kind=kind, passed=passed, r=recorded(r), rust=recorded(u)))

assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
Path("target/file-stream-parity.json").write_text(json.dumps(dict(
    r_version=r_version.stdout + r_version.stderr, rscript_version=version.stdout + version.stderr,
    reference_lc_ctype=locale,
    source_sha256=sources, binary_sha256=hashes, programs=cases,
    byte_programs=[dict(label=label, source=encoded(source), status=status, kind=kind)
                   for label, source, status, kind in byte_cases], cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"] and row["kind"] != "known"]
for row in failed:
    print(json.dumps(row))
print(f"{sum(row['passed'] for row in results if row['kind'] != 'known')} passing comparisons; "
      f"{sum(not row['passed'] for row in results if row['kind'] == 'known')} known byte-validation limits; {len(failed)} unexpected differences")
print("Success streams and status compare exactly. Error wording and Execution halted are not compared.")
if failed:
    raise SystemExit(1)
