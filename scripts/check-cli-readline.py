"""Compare readline batch behavior and live console input with R 4.6.0."""
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
rscript = binary.parent / "Rscript"
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent Rscript 4.6.0 reference is required")

cases = [("valid", code) for code in [
    "readline()", 'readline("ask: ")', 'readline(pr="partial: ")', "readline(prompt=)",
    'readline(c("one","two"))', "readline(NULL)", "readline(1)", "readline(NA_character_)",
    "readline(list(1,2))", "readline(list(NULL))", "readline(matrix(1:4,2))",
    'readline(factor("b",levels=c("a","b")))',
    'readline("previous");readline(as.character(NULL));readline(NULL)',
    'readline(strrep("p",300))', 'readline("α: ")',
    'withVisible(readline(prompt=invisible("p")))$visible',
    "readline(prompt=print(9))", '(function(...)readline(...))(pr="forward")',
    '(function()readline(prompt=return(7)))()',
    'invisible(lapply(c("a","b"),readline))',
]]
cases += [("error", code) for code in [
    "readline(bogus=print(9))", 'readline("p",print(9))',
    'readline(pr=print(9),prompt="p")', 'readline(prompt="p",prompt=print(9))',
    "readline(function()1)", "readline(,)",
]]
cases += [
    ("unsupported", "readline(list(1:2))"),
    ("unsupported", 'readline(strrep("é",128))'),
    ("format-limit", "readline(1e6)"),
]


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def console(command, scenario):
    master, slave = pty.openpty()
    settings = termios.tcgetattr(slave)
    settings[3] &= ~termios.ECHO
    termios.tcsetattr(slave, termios.TCSANOW, settings)
    child = subprocess.Popen(command, stdin=slave, stdout=slave, stderr=slave,
                             start_new_session=True)
    os.close(slave)
    transcript = bytearray()

    def wait_for(marker, start):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if marker and marker in transcript[start:]:
                return bytes(transcript[start:])
            if select.select([master], [], [], 0.1)[0]:
                try:
                    chunk = os.read(master, 65536)
                except OSError as error:
                    if error.errno == errno.EIO:
                        break
                    raise
                if not chunk:
                    break
                transcript.extend(chunk)
            elif child.poll() is not None:
                break
        if marker:
            raise AssertionError((command, marker, recorded(bytes(transcript))))
        return bytes(transcript[start:])

    def send(data, marker):
        start = len(transcript)
        os.write(master, data)
        return wait_for(marker, start).replace(b"\r\n", b"\n")

    try:
        wait_for(b"> ", 0)
        if scenario == "answers":
            phase = send(b'x<-readline("NAME? ");cat("VALUE:",x,":",interactive(),"\\n",sep="")\n',
                         b"NAME? ")
            assert b"VALUE:" not in phase, recorded(phase)
            phase = send(" \tα two words\t \n".encode(), b"> ")
            assert "VALUE:α two words:TRUE\n".encode() in phase, recorded(phase)
            phase = send(b'y<-readline("NEXT? ");cat("SECOND:",y,"\\n",sep="")\n'
                         b' buffered answer \nprint(7)\nq("no")\n', None)
            assert b"SECOND:buffered answer\n" in phase and b"[1] 7\n" in phase, recorded(phase)
        elif scenario == "eof":
            send(b'x<-readline("EOF? ");cat("EMPTY:",nchar(x),"\\n",sep="")\n', b"EOF? ")
            phase = send(b"\x04", b"> ")
            assert b"EMPTY:0\n" in phase, recorded(phase)
            send(b'q("no")\n', None)
        else:
            send(b'.Last<-function(){x<-readline("LAST? ");cat("LAST:",x,"\\n",sep="")};q()\n',
                 b"Save workspace image? [y/n/c]: ")
            phase = send(b"c\n", b"> ")
            assert b"LAST?" not in phase, recorded(phase)
            send(b"q()\n", b"Save workspace image? [y/n/c]: ")
            phase = send(b"n\nfinal reply\n", None)
            assert b"LAST? LAST:final reply\n" in phase, recorded(phase)
        status = child.wait(timeout=2)
        assert status == 0, (status, recorded(bytes(transcript)))
        return dict(command=command, scenario=scenario, status=status,
                    transcript=recorded(bytes(transcript)), passed=True)
    except Exception as error:
        return dict(command=command, scenario=scenario, status=child.poll(), passed=False,
                    error=str(error), transcript=recorded(bytes(transcript)))
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)


results = []
terminals = []
with tempfile.TemporaryDirectory(prefix="r-ust-readline-") as directory:
    snapshots = []
    for original in [binary, rscript]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {snapshot.name: hashlib.sha256(snapshot.read_bytes()).hexdigest()
              for snapshot in snapshots}
    for kind, code in cases:
        r = subprocess.run(["Rscript", "--vanilla", "-e", code], cwd=directory,
                           capture_output=True, timeout=20)
        for snapshot in snapshots:
            u = subprocess.run([str(snapshot), "-e", code], cwd=directory,
                               capture_output=True, timeout=20)
            if kind == "unsupported":
                passed = r.returncode == 0 and u.returncode != 0 and b"not implemented" in u.stderr
            elif kind == "format-limit":
                passed = r.returncode == u.returncode == 0 and r.stdout != u.stdout
            else:
                passed = r.returncode == u.returncode and r.stdout == u.stdout
                passed = passed and (r.returncode != 0 if kind == "error" else not r.stderr and not u.stderr)
            results.append(dict(kind=kind, code=code, binary=snapshot.name, passed=passed,
                                r_status=r.returncode, rust_status=u.returncode,
                                r_stdout=recorded(r.stdout), rust_stdout=recorded(u.stdout),
                                r_stderr=recorded(r.stderr), rust_stderr=recorded(u.stderr)))
    for scenario in ["answers", "eof", "save-cancel-last"]:
        reference = ["R", "--quiet", "--no-readline", "--no-environ", "--no-site-file",
                     "--no-init-file", "--no-restore"]
        native = [str(snapshots[0]), "--quiet"]
        if scenario != "save-cancel-last":
            reference.append("--no-save")
            native.append("--no-save")
        terminals.append(console(reference, scenario))
        terminals.append(console(native, scenario))

failed = [row for row in results if not row["passed"]]
terminal_failed = [row for row in terminals if not row["passed"]]
artifact = Path("target/cli-readline-parity.json")
artifact.parent.mkdir(parents=True, exist_ok=True)
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    binary_sha256=hashes, cases=results, terminals=terminals),
                               indent=2) + "\n")
for row in failed:
    print(json.dumps(row))
for row in terminal_failed:
    print(json.dumps(row))
unsupported = sum(row["kind"] in ("unsupported", "format-limit") for row in results)
print(f"{len(results)} batch readline probes; {unsupported} known limits (not passing); {len(failed)} mismatches")
print(f"{len(terminals)} terminal probes; {len(terminal_failed)} failures: live prompts, buffered answers, EOF, cancel, save-no, and .Last input.")
print("Batch streams compared; terminal assertions check protocol and values. Diagnostic wording not compared.")
if failed or terminal_failed:
    raise SystemExit(1)
