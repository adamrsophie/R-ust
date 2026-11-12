#!/usr/bin/env python3
import base64
import errno
import hashlib
import json
import os
from pathlib import Path
import pty
import select
import shutil
import subprocess
import sys
import tempfile
import termios
import time


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
environment = os.environ.copy()
environment["R_DEFAULT_PACKAGES"] = "NULL"


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command, code):
    result = subprocess.run(command, input=code.encode(), env=environment,
                            capture_output=True, timeout=30)
    return dict(status=result.returncode, stdout=recorded(result.stdout), stderr=recorded(result.stderr))


def terminal(command):
    master, slave = pty.openpty()
    settings = termios.tcgetattr(slave)
    settings[3] &= ~termios.ECHO
    termios.tcsetattr(slave, termios.TCSANOW, settings)
    child = subprocess.Popen(command, stdin=slave, stdout=slave, stderr=slave,
                             start_new_session=True, env=environment)
    os.close(slave)
    data = bytearray()
    try:
        os.write(master, b'cat(interactive());q("no")\n')
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if not select.select([master], [], [], 0.1)[0]:
                continue
            try:
                chunk = os.read(master, 65536)
            except OSError as error:
                if error.errno == errno.EIO:
                    break
                raise
            if not chunk:
                break
            data.extend(chunk)
        else:
            raise TimeoutError("terminal did not exit")
        return dict(status=child.wait(timeout=5), stdout=recorded(bytes(data).replace(b"\r\n", b"\n")))
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)


large = "#" + "x" * 10000
full = "#" + "x" * 9997
cases = [dict(label=f"bytes-{size}", expressions=["#" + "x" * (size - 1)])
         for size in [9997, 9998, 9999, 10000]]
cases += [dict(label=f"whitespace-{ord(character)}-{count}", expressions=["#" + character * count])
          for character in [" ", "\t", "\n"] for count in [3332, 3333]]
cases += [
    dict(label="full-rejects-later", expressions=[full, "cat('NEVER')"]),
    dict(label="full-rejects-two", expressions=[full, "cat('ONE')", "cat('TWO')"]),
    dict(label="exact-second-boundary", expressions=["#" + "x" * 9995, "1"]),
    dict(label="decoded-prior-length", expressions=["#" + " " * 3332, "#" + " " * 2221, "cat('A')"]),
    dict(label="literal-marker-compression", expressions=["#" + "~+~" * 3332, "#" + " " * 2221, "cat('A')"]),
    dict(label="omission-does-not-fill-buffer", expressions=["cat('FIRST')", large, "cat('LAST')"]),
    dict(label="omit-invalid-source", expressions=["@" + "x" * 10000]),
    dict(label="two-omissions", expressions=[large, "#" + "y" * 10000]),
    dict(label="metadata", expressions=["writeLines(commandArgs()[-1])", large], trailing=["literal", ""]),
    dict(label="fallback-args", expressions=[large], trailing=["--help", "-e", "print(99)"],
         stdin="writeLines(commandArgs(TRUE));cat(interactive())\n"),
    dict(label="warning-before-dynamic-echo", expressions=["options(echo=TRUE);cat('START')", large, "1"]),
    dict(label="warning-before-error", expressions=["cat('BEFORE');missingName", large], error=True),
]
for character, padding in [("é", 1120), ("é", 1119), ("€", 1119), ("😀", 1119)]:
    cases.append(dict(label=f"warning-utf8-{ord(character)}-{padding}", expressions=["#" + "x" * padding + character * 6000]))
for length in [9998, 9999]:
    code = 'cat("before\\n")\ns<-"\\u0061' + "x" * (length - 1) + '";cat(nchar(s))\n'
    cases.append(dict(label=f"unicode-parser-source-{length}", expressions=[code.replace("\\", "\\\\")], stdin=""))

sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
terminals = []
with tempfile.TemporaryDirectory(prefix="r-ust-expression-buffer-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    for name, reference, native in [
        ("R", ["R", "--vanilla", "--no-echo"], [str(snapshots[0]), "--vanilla", "--no-echo"]),
        ("Rscript", ["Rscript", "--vanilla"], [str(snapshots[1]), "--vanilla"]),
    ]:
        for case in cases:
            args = [argument for expression in case["expressions"] for argument in ["-e", expression]]
            if case.get("trailing"):
                args += (["--args"] if name == "R" else []) + case["trailing"]
            code = case.get("stdin", "cat('STDIN')\n")
            r, u = run(reference + args, code), run(native + args, code)
            passed = r == u and r["status"] == 0
            if case.get("error"):
                passed = (r["status"] == u["status"] == 1 and r["stdout"] == u["stdout"]
                          and bool(r["stderr"]["text"]) and bool(u["stderr"]["text"]))
            results.append(dict(frontend=name, case=case, r=r, rust=u, passed=passed))
        r = terminal(reference + ["-e", large])
        u = terminal(native + ["-e", large])
        terminals.append(dict(frontend=name, r=r, rust=u, passed=(r == u and r["status"] == 0
                                                               and r["stdout"]["text"].endswith("FALSE"))))
        print(f"Checked {name}: {len(cases)} batch cases and terminal fallback", flush=True)
    path = Path(directory) / "input.R"
    path.write_text("cat('FILE')\n")
    for args in [["-f", str(path), "-e", large], ["-e", large, "-f", str(path)]]:
        r = run(["R", "--vanilla", "--no-echo", *args], "cat('STDIN')\n")
        u = run([str(snapshots[0]), "--vanilla", "--no-echo", *args], "cat('STDIN')\n")
        results.append(dict(frontend="R", case=dict(label="file-with-omitted-expression", args=args),
                            r=r, rust=u, passed=(r == u and r["status"] == 0 and r["stdout"]["text"].endswith("FILE"))))

Path("target/expression-buffer-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_default_packages="NULL", source_sha256=sources,
    binary_sha256=hashes, cases=results, terminals=terminals), indent=2) + "\n")
failed = [row for row in results + terminals if not row["passed"]]
for row in failed:
    print(json.dumps(dict(frontend=row["frontend"], label=row.get("case", {}).get("label", "terminal"),
                         r_status=row["r"]["status"], rust_status=row["rust"]["status"],
                         r_stdout_bytes=len(base64.b64decode(row["r"]["stdout"]["base64"])),
                         rust_stdout_bytes=len(base64.b64decode(row["rust"]["stdout"]["base64"])))))
print(f"{len(results)} batch comparisons; {len(terminals)} terminal comparisons; {len(failed)} differences")
print("Exact streams/status compared except diagnostic wording in the explicit error case.")
if failed:
    raise SystemExit(1)
