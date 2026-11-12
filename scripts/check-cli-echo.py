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
codes = [
    "1", "1\n", "", "\n", "# comment", "# comment\n1\n", "\n\n1\n\n",
    "1;2\n", "1+\n2\n", "sum(\n1,\n2\n)\n", "{\n1;\n2\n}\n",
    "f<-function(x,\ny=2) {\nx+y\n}\nf(3)\n",
    'writeLines("first\nsecond")\n', 'writeLines("α")\n',
    "1\r\n2\r\n", "1+\r\n2", "1\nmissingName\n9\n", "1\n@\n9\n",
    "1+\n", "print(1);q('no');@\n9\n", "q('no',status=3)\n9\n",
    "options(echo=FALSE)\n1\n", "options(echo=TRUE)\n1\n",
    "options(echo=NA)\n1\n", "options(echo=FALSE);1;2\n3\n",
    "options(echo=TRUE);1;2\n3\n",
    'options(prompt="P? ",continue="C? ")\n1+\n2\n',
    'options(prompt="P? ",continue="C? "); (\n4)\n',
    '.Last<-function()print(8)\n1\n',
    '.Last<-function()options(echo=TRUE)\noptions(echo=FALSE)\n1\n',
    "f<-function()options(echo=FALSE)\nf()\n1\n",
]


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def compare(reference, native, code, mode, no_echo, directory):
    r = subprocess.run(reference, input=code.encode() if mode == "stdin" else None,
                       cwd=directory, env=environment, capture_output=True, timeout=20)
    u = subprocess.run(native, input=code.encode() if mode == "stdin" else None,
                       cwd=directory, env=environment, capture_output=True, timeout=20)
    row = dict(code=code, mode=mode, no_echo=no_echo,
               passed=(r.returncode == u.returncode and r.stdout == u.stdout
                       and bool(r.stderr) == bool(u.stderr)),
               r_status=r.returncode, rust_status=u.returncode,
               r_stdout=recorded(r.stdout), rust_stdout=recorded(u.stdout),
               r_stderr=recorded(r.stderr), rust_stderr=recorded(u.stderr))
    return r, u, row


def console(command):
    master, slave = pty.openpty()
    settings = termios.tcgetattr(slave)
    settings[3] &= ~termios.ECHO
    termios.tcsetattr(slave, termios.TCSANOW, settings)
    child = subprocess.Popen(command, stdin=slave, stdout=slave, stderr=slave,
                             start_new_session=True, env=environment)
    os.close(slave)
    transcript = bytearray()

    def wait_for(marker, start):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if marker and marker in transcript[start:]:
                return bytes(transcript[start:]).replace(b"\r\n", b"\n")
            if select.select([master], [], [], 0.1)[0]:
                try:
                    data = os.read(master, 65536)
                except OSError as error:
                    if error.errno == errno.EIO:
                        break
                    raise
                if not data:
                    break
                transcript.extend(data)
            elif child.poll() is not None:
                break
        if marker:
            raise AssertionError((marker.decode(), recorded(bytes(transcript))))
        return bytes(transcript[start:]).replace(b"\r\n", b"\n")

    def send(code, marker):
        start = len(transcript)
        os.write(master, code)
        return wait_for(marker, start)

    try:
        phase = send(b'cat("READY\\n")\n', b"READY\r\n")
        assert b"> " not in phase, recorded(phase)
        send(b'options(echo=TRUE,prompt="LIVE? ",continue="MORE? ")\n', b"LIVE? ")
        send(b'1+\n', b"MORE? ")
        phase = send(b'2\n', b"LIVE? ")
        assert b"[1] 3\n" in phase, recorded(phase)
        phase = send(b'options(echo=FALSE);cat("OFF\\n")\n', b"OFF\r\n")
        assert b"LIVE? " not in phase, recorded(phase)
        phase = send(b'cat("STILL OFF\\n")\n', b"STILL OFF\r\n")
        assert b"LIVE? " not in phase, recorded(phase)
        phase = send(b'options(echo=NA)\n', b"LIVE? ")
        send(b'q()\n', None)
        status = child.wait(timeout=3)
        assert status == 0, status
        return dict(command=command, passed=True, status=status,
                    transcript=recorded(bytes(transcript)))
    except Exception as error:
        return dict(command=command, passed=False, error=str(error),
                    transcript=recorded(bytes(transcript)))
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)


results = []
source_hashes = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                              Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
with tempfile.TemporaryDirectory(prefix="r-ust-echo-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "input.R"
    for index, code in enumerate(codes):
        path.write_bytes(code.encode())
        for mode in ["file", "stdin", "expression", "rscript"]:
            if mode in ("expression", "rscript") and not code.strip():
                continue
            for no_echo in [False, True]:
                if mode == "rscript" and not no_echo:
                    continue
                reference = ["R", "--vanilla", "--quiet", "--no-readline"]
                native = [str(snapshots[0]), "--quiet", "--no-save"]
                if no_echo:
                    reference.append("--no-echo")
                    native.append("--no-echo")
                if mode == "file":
                    reference += ["-f", str(path)]
                    native += ["-f", str(path)]
                elif mode == "expression":
                    reference += ["-e", code]
                    native += ["-e", code]
                elif mode == "rscript":
                    reference = ["Rscript", "--vanilla", "-e", code]
                    native = [str(snapshots[1]), "-e", code]
                _, _, row = compare(reference, native, code, mode, no_echo, directory)
                results.append(row)
        if (index + 1) % 10 == 0:
            print(f"Checked {index + 1}/{len(codes)} programs", flush=True)
    limits = []
    code = r'options(echo=TRUE);cat("VALUE\n")'
    for mode, reference, native in [
        ("expression", ["R", "--vanilla", "--no-echo", "-e", code],
         [str(snapshots[0]), "--no-echo", "-e", code]),
        ("rscript", ["Rscript", "--vanilla", "-e", code], [str(snapshots[1]), "-e", code]),
    ]:
        r, u, row = compare(reference, native, code, mode, True, directory)
        row.update(kind="launcher-escape")
        results.append(row)
    code = 'nchar("' + 'x' * 5000 + '")\n'
    path.write_text(code)
    for mode in ["file", "stdin"]:
        arguments = ["-f", str(path)] if mode == "file" else []
        r, u, row = compare(["R", "--vanilla", "--quiet", *arguments],
                            [str(snapshots[0]), "--quiet", "--no-save", *arguments],
                            code, mode, False, directory)
        row.update(kind="console-buffer")
        results.append(row)
    terminals = [console(["R", "--vanilla", "--no-readline", "--no-echo"]),
                 console([str(snapshots[0]), "--no-echo"])]

artifact = Path("target/cli-echo-parity.json")
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    reference_default_packages="NULL", source_sha256=source_hashes,
                                    binary_sha256=hashes, cases=results, terminals=terminals,
                                    known_limits=limits), indent=2) + "\n")
failed = [row for row in results + terminals if not row["passed"]]
failed += [row for row in limits if not row["limit_observed"]]
for row in failed:
    print(json.dumps(row))
print(f"{len(results)} batch echo comparisons; {len(terminals)} terminal checks; {len(failed)} failures")
print(f"{sum(row['limit_observed'] for row in limits)} known console-buffer limits observed, not counted as passing")
print("Stdout, statuses and diagnostic presence compared; diagnostic wording and terminal input echo not compared.")
if failed:
    raise SystemExit(1)
