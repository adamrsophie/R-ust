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


class Result(TypedDict):
    status: int
    stdout: str
    stderr: str


class Snapshot(TypedDict):
    stdout: str
    stderr: str
    running: bool
    matched: bool


class StreamResult(Result):
    phases: list[Snapshot]


class Comparison(TypedDict):
    label: str
    mode: str
    echo: bool
    passed: bool
    r: Result
    rust: Result


def encoded(data: bytes) -> str:
    return base64.b64encode(data).decode("ascii")


def recorded(result: subprocess.CompletedProcess[bytes]) -> Result:
    return Result(status=result.returncode, stdout=encoded(result.stdout), stderr=encoded(result.stderr))


def padding(size: int) -> str:
    assert size >= 6
    return 's<-"' + 'x' * (size - 6) + '";'


def split_after(expression: str, prefix: str) -> str:
    assert expression.startswith(prefix)
    return padding(4095 - len(prefix.encode())) + expression


def streamed(command: list[str], fifo: Path, phases: list[tuple[bytes, bytes]]) -> StreamResult:
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
        for source, expected in phases:
            assert os.write(writer, source) == len(source)
            deadline = time.monotonic() + 10
            while len(output) < len(expected) and process.poll() is None and time.monotonic() < deadline:
                drain(0.05)
            drain(0.1)
            states.append(Snapshot(stdout=encoded(bytes(output)), stderr=encoded(bytes(errors)),
                                   running=process.poll() is None,
                                   matched=bytes(output) == expected and not errors))
        os.close(writer)
        writer = None
        tail, stderr = process.communicate(timeout=20)
        assert process.returncode is not None
        return StreamResult(status=process.returncode, stdout=encoded(bytes(output) + tail),
                            stderr=encoded(bytes(errors) + stderr), phases=states)
    finally:
        if writer is not None:
            os.close(writer)
        if process.poll() is None:
            process.kill()
            _ = process.wait(timeout=10)


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
reference_r = shutil.which("R")
reference_script = shutil.which("Rscript")
if not reference_r or not reference_script or Path(reference_r).resolve() == binary:
    raise SystemExit("Independent R/Rscript 4.6.0 executables are required")
versions = [subprocess.run([program, "--version"], capture_output=True, text=True, check=True)
            for program in [reference_r, reference_script]]
if any("version 4.6.0" not in result.stdout + result.stderr for result in versions):
    raise SystemExit("Reference R and Rscript must be version 4.6.0")
locale = subprocess.run([reference_script, "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True).stdout
programs: list[tuple[str, str]] = []
for length in [4084, 4085, 4086, 5000, 8180, 8181, 9000]:
    for ending in ["", "\n"]:
        programs.append((f"literal-{length}-{'lf' if ending else 'eof'}", f'nchar("{"x" * length}")' + ending))
for unit, lengths in [("α", [4086, 4087, 4088]), ("😀", [4085, 4086, 4087, 4088])]:
    for length in lengths:
        programs.append((f"utf8-{ord(unit)}-{length}", f'nchar("{"x" * length}{unit}xx")\n'))
for label, expression, prefix in [
    ("integer", "12\n", "1"), ("decimal", "1e2\n", "1e"),
    ("decimal-sign", "1e+2\n", "1e+"), ("hex", "0x10\n", "0x"),
    ("hex-exponent", "0x1p+2\n", "0x1p"), ("hex-sign", "0x1p+2\n", "0x1p+"),
    ("superassign", "x<<-1\ncat(x)\n", "x<<"),
    ("block", "{1;2}\n", "{1"), ("call", 'nchar("a")\n', 'nchar("a"'),
    ("function", "f<-function(x) x\nf(1)\n", "f<-function(x"),
]:
    programs.append((label, split_after(expression, prefix)))
for expression in ["{1}", 'nchar("a")', "c(1)[1]", "if(TRUE)1"]:
    programs.append((f"closed-{expression}", split_after(expression + ';cat("END")\n', expression)))
for prefix in ['nchar(r', 'nchar(r"', 'nchar(r"-', 'nchar(r"---', 'nchar(r"---(v)', 'nchar(r"---(v)--']:
    programs.append((f"raw-{len(prefix)}", split_after('nchar(r"---(v)---")\n', prefix)))
for escape in ["\\x41", "\\u0061", "\\101"]:
    expression = f'nchar("{escape}")\n'
    for count in range(1, len(escape)):
        prefix = 'nchar("' + escape[:count]
        programs.append((f"escape-{escape}-{count}", split_after(expression, prefix)))
programs.extend([
    ("comment", "#" + "x" * 5000 + '\ncat("END")\n'),
    ("whitespace", " " * 5000 + "1\n"),
    ("full-number-eof", " " * 4094 + "1"),
    ("full-comment-eof", "#" + "x" * 4094),
    ("full-whitespace-eof", " " * 4095),
    ("split-crlf", padding(4093) + "1\r\n"),
    ("whole-crlf", padding(4092) + "1\r\n"),
    ("invalid-last-byte", padding(4094) + "@\n"),
    ("leading-semicolon", ";1\n"), ("repeated-semicolon", "1;;2\n"),
    ("block-semicolon", "{;1}\n"),
    ("block-repeated-semicolon", "{;;1;;2;}\n"),
    ("comment-cr-eof", "# comment\r"),
])
sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results: list[Comparison] = []
omitted: list[str] = []
with tempfile.TemporaryDirectory(prefix="r-ust-console-buffer-") as directory:
    root = Path(directory)
    snapshots: list[Path] = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = root / original.name
        _ = shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    for echo in [False, True]:
        for label, program in programs:
            source = ('options(echo=TRUE)\n' if echo else '') + program
            path = root / "input.R"
            _ = path.write_text(source)
            expression = source.replace("\\", "\\\\")
            argument = expression.replace(" ", "~+~").replace("\n", "~n~").replace("\t", "~t~")
            for mode in ["stdin", "file", "expression", "rscript-file", "rscript-expression"]:
                if mode.endswith("expression") and len(argument.encode()) + 2 > 10000:
                    omitted.append(f"{label}/{mode}/echo={echo}")
                    continue
                reference, native = ([reference_script, "--vanilla"], [str(snapshots[1]), "--vanilla"]) if mode.startswith("rscript") else (
                    [reference_r, "--vanilla", "--no-echo"], [str(snapshots[0]), "--vanilla", "--no-echo"])
                args = ["-e", expression] if mode.endswith("expression") else ([str(path)] if mode == "rscript-file" else ["-f", str(path)])
                if mode == "stdin":
                    args = []
                r, u = [subprocess.run([*command, *args], input=source.encode() if mode == "stdin" else b"",
                                      env=environment, capture_output=True, timeout=20) for command in [reference, native]]
                passed = r.returncode == u.returncode and r.stdout == u.stdout
                passed = passed and (r.stderr == u.stderr if r.returncode == 0 else bool(r.stderr) and bool(u.stderr))
                results.append(Comparison(label=label, mode=mode, echo=echo, passed=passed, r=recorded(r), rust=recorded(u)))
        print(f"Checked {len(programs)} programs, echo={echo}", flush=True)
    fifo = root / "input.fifo"
    os.mkfifo(fifo)
    for mode, reference, native in [
        ("live-main", [reference_r, "--vanilla", "--no-echo", "-f"], [str(snapshots[0]), "--vanilla", "--no-echo", "-f"]),
        ("live-rscript", [reference_script, "--vanilla"], [str(snapshots[1]), "--vanilla"]),
    ]:
        for echo in [False, True]:
            for unit, cut in [(b"z", 1), ("α".encode(), 1), ("😀".encode(), 2)]:
                prefix = b'cat("READY\\n");nchar("'
                filler = 4095 - len(prefix) - cut
                source = prefix + b"x" * filler + unit + b'xx")\ncat("TAIL\\n")\n'
                first, tail = source[:4095], source[4095:]
                start = f"options(echo={'TRUE' if echo else 'FALSE'})\n".encode()
                before = (b"> " + first + b"READY\n+ ") if echo else b"READY\n"
                after = before + (tail.split(b"\n", 1)[0] + b"\n" if echo else b"")
                after += f"[1] {filler + 3}\n".encode()
                after += b'> cat("TAIL\\n")\nTAIL\n> ' if echo else b"TAIL\n"
                phases = [(start + first, before), (tail, after)]
                r, u = [streamed([*command, str(fifo)], fifo, phases) for command in [reference, native]]
                passed = r["status"] == u["status"] == 0 and r["stdout"] == u["stdout"] and r["stderr"] == u["stderr"] == ""
                passed = passed and all(len(run["phases"]) == 2 and all(state["matched"] and state["running"] for state in run["phases"]) for run in [r, u])
                results.append(Comparison(label=f"split-{unit.hex()}", mode=mode, echo=echo, passed=passed, r=r, rust=u))
        print(f"Checked {mode}: 6 live chunk scenarios", flush=True)

assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
_ = Path("target/console-buffer-parity.json").write_text(json.dumps(dict(
    versions=[result.stdout + result.stderr for result in versions], reference_lc_ctype=locale,
    source_sha256=sources, binary_sha256=hashes, programs=programs,
    omitted_expression_cases=omitted, cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"]]
for row in failed:
    print(f"FAIL {row['label']} {row['mode']} echo={row['echo']}")
print(f"{len(results) - len(failed)} passing comparisons; {len(failed)} unexpected differences; {len(omitted)} expression cases excluded by launcher size limit")
print("Raw successful streams and status compare exactly; error wording and Execution halted are excluded.")
if failed:
    raise SystemExit(1)
