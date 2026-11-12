#!/usr/bin/env python3
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import select
import shutil
import subprocess
import sys
import tempfile
import time
from typing import NotRequired, TypedDict


class Case(TypedDict):
    label: str
    source: str
    status: int
    expected: NotRequired[str]
    order: NotRequired[bool]
    modes: NotRequired[list[str]]


class Recorded(TypedDict):
    text: str
    base64: str


class LiveResult(TypedDict):
    before: Recorded
    waiting: bool
    status: int
    merged: Recorded


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
locale = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True).stdout
cases: list[Case] = []
literals = [("1L", "1", False), ("1.0L", "1", True), ("1.5L", "1.5", True),
            ("1e-1L", "0.1", True), ("1.5e0L", "1.5", True), ("2147483648L", "2147483648", True),
            ("-2147483648L", "-2147483648", True), ("0x1L", "1", False), ("0x1.0L", "1", True),
            ("0x1.8L", "1.5", True), ("0x80000000L", "2147483648", True), ("0x1p-1L", "0.5", True),
            ("0x1.0p0L", "1", False), ("0x.p1L", "0", False), ("0x.L", "0", True)]
for level in [-1, 0, 1, 2, 3]:
    for literal, value, warning in literals:
        status = 1 if level >= 2 and warning else 0
        cases.append(Case(label=f"literal-{level}-{literal}",
                          source=f'options(warn={level})\ncat("BEFORE\\n")\ncat({literal},"\\n",sep="")\ncat("AFTER\\n")\n',
                          expected="BEFORE\n" if status else f"BEFORE\n{value}\nAFTER\n", status=status,
                          order=literal in ["1.5L", "0x1L"]))
for level in [0, 1, 2]:
    for suffix in ["+@", "+", "+\n2.5L+\n3"]:
        complete = suffix.endswith("3")
        status = 1 if not complete or level == 2 else 0
        cases.append(Case(label=f"continuation-{level}-{suffix}",
                          source=f'options(warn={level})\ncat("BEFORE\\n")\n1.5L{suffix}\n',
                          expected="BEFORE\n[1] 7\n" if status == 0 else "BEFORE\n", status=status, order=True))
    for length, zeros in [(100, 120), (1000, 1200)]:
        cases.append(Case(label=f"truncate-{level}-{length}",
                          source=f'options(warn={level},warning.length={length})\n0.' + '0' * zeros + 'L\n',
                          expected="" if level == 2 else "[1] 0\n", status=1 if level == 2 else 0))
for count in [1, 2, 10, 11, 49, 50, 51]:
    cases.append(Case(label=f"count-{count}", source='{' + '1.5L;' * count + 'cat("COUNT\\n")}\n',
                      expected="COUNT\n", status=0))
for capacity in [1, 2, 10, 11, 50]:
    cases.append(Case(label=f"capacity-{capacity}",
                      source=f'options(nwarnings={capacity})\n' + '{' + '1.5L;' * 51 + 'cat("COUNT\\n")}\n',
                      expected="COUNT\n", status=0))
cases += [
    Case(label="changed-option", source='{options(warn=-1);cat(1.5L,"VALUE\\n")}\n', status=0, order=True),
    Case(label="changed-immediate", source='{options(warn=1);cat(1.5L,"VALUE\\n")}\n', status=0, order=True),
    Case(label="unreached", source='if(FALSE)1.5L\n', expected="", status=0, order=True),
    Case(label="definition", source='f<-function()1.5L;cat("DEFINED\\n");cat(f())\n', expected="DEFINED\n1.5", status=0, order=True),
    Case(label="evaluation-error", source='{cat("INSIDE\\n");1.5L;missingName}\n', expected="INSIDE\n", status=1, order=True),
    Case(label="quit", source='{cat("INSIDE\\n");1.5L;q("no")}\n', expected="INSIDE\n", status=0, order=True),
    Case(label="last", source='.Last<-function()cat("LAST\\n");{1.5L;q("no")}\n', expected="LAST\n", status=0, order=True),
    Case(label="last-error", source='.Last<-function()missingName;{1.5L;q("no")}\n', expected="", status=1, order=True),
    Case(label="quoted-comment", source='cat("1.5L\\n");# 1.5L\n', expected="1.5L\n", status=0),
    Case(label="empty", source="\n", expected="", status=0, modes=["stdin", "rscript-file"]),
    Case(label="empty-rscript-expression", source="\n", expected="", status=1, modes=["rscript-expression"]),
    Case(label="empty-expression", source="\n", expected="", status=1, modes=["expression"]),
]


def recorded(data: bytes) -> Recorded:
    return Recorded(text=data.decode("utf-8", errors="replace"), base64=base64.b64encode(data).decode("ascii"))


def run(command, data, merged=False):
    if merged:
        result = subprocess.run(command, input=data, env=environment, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, timeout=30)
    else:
        result = subprocess.run(command, input=data, env=environment, capture_output=True, timeout=30)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout), stderr=recorded(result.stderr or b""))


def normalized(data):
    lines = []
    for line in data.splitlines(keepends=True):
        if line == b"Execution halted\n":
            continue
        marker = b"(converted from warning) "
        if marker in line:
            lines.append(marker + line.split(marker, 1)[1])
        elif line.startswith((b"Error", b"parse error:", b"domain error:", b"object '", b"type error:", b"argument error:")):
            lines.append(b"Error\n")
        else:
            lines.append(line)
    return b"".join(lines)


def warning_lines(data):
    lines = []
    for line in data.splitlines(keepends=True):
        if b"(converted from warning) " in line:
            lines.append(line.split(b"(converted from warning) ", 1)[1])
        elif line.startswith((b"Warning", b"In addition:", b"integer literal ", b"non-integer value ", b"There were ")) or re.match(rb"\d+: ", line):
            lines.append(line)
    return b"".join(lines)


def live(command: list[str], level: int) -> tuple[bytes, LiveResult]:
    process = subprocess.Popen(command, env=environment, stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, bufsize=0)
    assert process.stdin is not None and process.stdout is not None
    before = b""
    try:
        process.stdin.write(f'options(warn={level});cat("READY\\n")\n1.5L+\n'.encode())
        process.stdin.flush()
        deadline = time.monotonic() + 30
        while b"READY\n" not in before and time.monotonic() < deadline:
            if select.select([process.stdout], [], [], 0.1)[0]:
                part = os.read(process.stdout.fileno(), 4096)
                if not part:
                    break
                before += part
        if select.select([process.stdout], [], [], 0.25)[0]:
            before += os.read(process.stdout.fileno(), 4096)
        waiting = process.poll() is None
        tail, _ = process.communicate(b'2\ncat("DONE\\n")\n', timeout=30)
        data = before + tail
        assert process.returncode is not None
        return data, LiveResult(before=recorded(before), waiting=waiting, status=process.returncode, merged=recorded(data))
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=10)


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results, orders, live_results = [], [], []
with tempfile.TemporaryDirectory(prefix="r-ust-numeric-warnings-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "warnings.R"
    for mode in ["stdin", "rscript-file", "expression", "rscript-expression"]:
        count = 0
        for case in cases:
            if mode not in case.get("modes", ["stdin", "rscript-file", "expression", "rscript-expression"]):
                continue
            source = case["source"].encode()
            reference = ["R", "--vanilla", "--no-echo"]
            native = [str(snapshots[0]), "--vanilla", "--no-echo"]
            data = source
            if mode == "rscript-file":
                path.write_bytes(source)
                reference = ["Rscript", "--vanilla", str(path)]
                native = [str(snapshots[1]), "--vanilla", str(path)]
                data = b""
            elif mode in ["expression", "rscript-expression"]:
                expression = case["source"].replace("\\", "\\\\")
                encoded = expression.replace(" ", "~+~").replace("\t", "~t~").replace("\n", "~n~")
                assert len(encoded.encode()) + 2 <= 10000
                if mode == "rscript-expression":
                    reference = ["Rscript", "--vanilla"]
                    native = [str(snapshots[1]), "--vanilla"]
                reference += ["-e", expression]
                native += ["-e", expression]
                data = b""
            r, rr = run(reference, data)
            u, uu = run(native, data)
            expected = case.get("expected")
            passed = r.returncode == u.returncode == case["status"] and r.stdout == u.stdout
            passed = passed and (expected is None or r.stdout == expected.encode())
            passed = passed and (r.stderr == u.stderr if case["status"] == 0
                                 else bool(r.stderr) and bool(u.stderr) and warning_lines(r.stderr) == warning_lines(u.stderr))
            results.append(dict(mode=mode, case=case, r=rr, rust=uu, passed=passed))
            count += 1
            if case.get("order", False):
                r, rr = run(reference, data, merged=True)
                u, uu = run(native, data, merged=True)
                passed = r.returncode == u.returncode == case["status"] and normalized(r.stdout) == normalized(u.stdout)
                orders.append(dict(mode=mode, label=case["label"], r=rr, rust=uu, passed=passed))
        print(f"Checked {mode}: {count} programs", flush=True)
    for level in [1, 2]:
        r, rr = live(["R", "--vanilla", "--no-echo"], level)
        u, uu = live([str(snapshots[0]), "--vanilla", "--no-echo"], level)
        passed = rr["before"]["text"] == uu["before"]["text"] == "READY\n" and rr["waiting"] and uu["waiting"]
        passed = passed and rr["status"] == uu["status"] == (0 if level == 1 else 1) and normalized(r) == normalized(u)
        live_results.append(dict(warn=level, r=rr, rust=uu, passed=passed))

Path("target/numeric-warning-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_lc_ctype=locale, source_sha256=sources,
    binary_sha256=hashes, cases=results, order_cases=orders, live_cases=live_results), indent=2) + "\n")
failed = [row for row in results if not row["passed"]]
failed += [row for row in [*orders, *live_results] if not row["passed"]]
for row in failed[:12]:
    case = row.get("case")
    label = case.get("label") if isinstance(case, dict) else row.get("label", row.get("warn"))
    print(json.dumps(dict(mode=row.get("mode"), label=label, passed=False)))
print(f"{sum(row['passed'] for row in results)} stream/status, {len(orders)} merged-order, {len(live_results)} live checks; "
      f"{len(failed)} unexpected differences")
print("Successful streams and warning text are exact; error wording and Execution halted are not compared.")
if failed:
    raise SystemExit(1)
