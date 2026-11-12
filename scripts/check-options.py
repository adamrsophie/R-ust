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
    raise SystemExit("An independent Rscript 4.6.0 reference is required")

cases = [("valid", code) for code in [
    "getOption('width')", "getOption('absent')", "getOption('absent',9)",
    "getOption('width',undefinedDefault)", "getOption(def=9,x='absent')",
    "getOption(NA_character_,9)", "getOption(matrix('width',1))",
    "withVisible(getOption('absent',invisible(9)))$visible",
    "withVisible(getOption(invisible('width')))$visible",
    "withVisible(options())$visible", "withVisible(options(NULL))$visible",
    "withVisible(options(list()))$visible", "withVisible(options(a=1))$visible",
    "withVisible(options(a=1,'width'))$visible",
    "identical(options(a=1,a=2),list(a=NULL,a=1))",
    "identical(options('absent','width'),list(absent=NULL,width=80L))",
    "identical(options(c('width','digits')),list(width=80L))",
    "identical(options(list('width',a=1)),list(width=80L,a=NULL))",
    "options(a=1);old<-options(a=2,b=3);options(old);identical(options('a','b'),list(a=1,b=NULL))",
    "options(a=1);saved<-options();options(a=2);saved$a",
    "options(a=1);options(a=NULL);getOption('a',9)",
    "options(a=list(x=2));getOption('a')$x",
    "options(a=function(x)x+1);f<-getOption('a');f(3)",
    "options(a=c(NA_real_,NaN));identical(getOption('a'),c(NA_real_,NaN))",
    "(function(...)options(...))(answer=4);getOption('answer')",
    "(function(...)getOption(...))('width',undefinedDefault)",
    "(function()getOption('absent',return(7)))()",
    "(function()getOption(x=return(8)))()",
    "invisible(lapply(c('width','digits'),getOption))",
    "options(answer=1);f<-function()options(answer=2);f();getOption('answer')",
]]

values = ["NULL", "TRUE", "FALSE", "NA", "NA_real_", "1L", "7.9", "80", "1000",
          "'80'", "'bad'", "''", "as.numeric(NULL)", "as.character(NULL)", "list(80)", "c(80,81)"]
names = ["width", "digits", "expressions", "deparse.cutoff", "max.print", "warning.length",
         "nwarnings", "showNCalls", "max.contour.segments", "prompt", "continue", "quiet",
         "echo", "check.bounds", "keep.source", "warn", "contrasts", "matprod"]
for name in names:
    for value in values:
        kind = "known-echo-limit" if name == "echo" and value in ("TRUE", "NA") else "validation"
        if name == "deparse.cutoff" and value in ("FALSE", "NA", "NA_real_", "'bad'", "''", "as.numeric(NULL)", "as.character(NULL)", "list(80)"):
            kind = "known-print-limit"
        code = f"options({name}={value});v<-getOption('{name}');cat(typeof(v),':',v,'\\n',sep='')"
        if kind == "known-echo-limit":
            code = f"options(echo={value});print(typeof(getOption('echo')));getOption('echo')"
        cases.append((kind, code))

for name in ["verbose", "show.error.messages", "catch.script.errors", "warnPartialMatchDollar",
             "warnPartialMatchArgs", "warnPartialMatchAttr", "showWarnCalls", "showErrorCalls",
             "browserNLdisabled", "CBoundsCheck", "stringsAsFactors"]:
    for value in ["TRUE", "FALSE", "NA", "1", "NULL"]:
        cases.append(("validation", f"options({name}={value});getOption('{name}')"))
cases += [("valid", code) for code in [
    "options(prompt=NA_real_);getOption('prompt')",
    "options(prompt=factor('b',levels=c('a','b')));getOption('prompt')",
    "options(prompt=c('α: ','ignored'));getOption('prompt')",
    "options(contrasts=c('contr.treatment','contr.poly'));identical(getOption('contrasts'),c('contr.treatment','contr.poly'))",
    "options(keep.parse.data=3);getOption('keep.parse.data')",
    "options(editor=c('one','two'));getOption('editor')",
    "options(scipen=NA);getOption('scipen')", "options(scipen=10000);getOption('scipen')",
    "options(PCRE_use_JIT=NA);getOption('PCRE_use_JIT')",
    "options(PCRE_limit_recursion=NA);getOption('PCRE_limit_recursion')",
]]
cases += [("error", code) for code in [
    "getOption()", "getOption(1)", "getOption('')", "getOption(as.character(NULL))",
    "getOption(c('width','digits'))", "getOption(bogus=print(9))",
    "getOption('width',1,print(9))", "getOption(x='width',x=print(9))",
    "getOption(def=print(9))", "options(1)", "options(list(1))",
    "options('')", "options(as.character(NULL))", "options(par.ask.default=TRUE)",
    "options('par.ask.default')", "options(width=print(2),answer=print(3))",
]]
cases.append(("known-print-limit", "options(deparse.cutoff=NA);print(1)"))


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def terminal(command):
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
                return bytes(transcript[start:]).replace(b"\r\n", b"\n")
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
            raise AssertionError((marker.decode(), recorded(bytes(transcript))))
        return bytes(transcript[start:]).replace(b"\r\n", b"\n")

    def send(data, marker):
        start = len(transcript)
        os.write(master, data)
        return wait_for(marker, start)

    try:
        wait_for(b"> ", 0)
        send(b'options(prompt="PRIMARY? ",continue="MORE? ")\n', b"PRIMARY? ")
        send(b'1+\n', b"MORE? ")
        phase = send(b'2\n', b"PRIMARY? ")
        assert b"[1] 3\n" in phase, recorded(phase)
        phase = send(b'options(prompt="AFTER? ",width=2)\n', b"\nAFTER? ")
        assert b"Error" in phase, recorded(phase)
        phase = send(b'cat("WIDTH:",getOption("width"),"\\n",sep="")\n', b"AFTER? ")
        assert b"WIDTH:80\n" in phase, recorded(phase)
        send(b'options(prompt="NEXT? ",continue="WAIT? "); (\n', b"WAIT? ")
        phase = send(b'4)\n', b"NEXT? ")
        assert b"[1] 4\n" in phase, recorded(phase)
        send(b'options(prompt="> ",continue="+ ")\n', b"> ")
        phase = send(b'q("no")\n', None)
        status = child.wait(timeout=3)
        assert status == 0, (status, recorded(phase))
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
with tempfile.TemporaryDirectory(prefix="r-ust-options-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    for index, (kind, code) in enumerate(cases):
        reference = subprocess.run(["Rscript", "--vanilla", "--default-packages=NULL", "-e", code], cwd=directory,
                                   capture_output=True, timeout=20)
        for snapshot in snapshots:
            native = subprocess.run([str(snapshot), "-e", code], cwd=directory,
                                    capture_output=True, timeout=20)
            passed = (reference.returncode == native.returncode
                      and reference.stdout == native.stdout)
            if reference.returncode == 0:
                passed = passed and bool(reference.stderr) == bool(native.stderr)
            if kind == "error":
                passed = passed and reference.returncode != 0
            known_limit = kind.startswith("known-")
            limit_observed = False
            if kind == "known-echo-limit":
                limit_observed = (reference.returncode == native.returncode == 0
                                  and reference.stdout == native.stdout + b"> \n"
                                  and not reference.stderr and not native.stderr)
            elif kind == "known-print-limit":
                limit_observed = (reference.returncode == native.returncode == 0
                                  and reference.stdout == native.stdout
                                  and b"invalid 'deparse.cutoff'" in reference.stderr
                                  and b"invalid 'deparse.cutoff'" not in native.stderr)
            if limit_observed:
                passed = False
            results.append(dict(kind=kind, code=code, binary=snapshot.name, passed=passed,
                                known_limit=known_limit, limit_observed=limit_observed,
                                r_status=reference.returncode, rust_status=native.returncode,
                                r_stdout=recorded(reference.stdout), rust_stdout=recorded(native.stdout),
                                r_stderr=recorded(reference.stderr), rust_stderr=recorded(native.stderr)))
        if (index + 1) % 100 == 0:
            print(f"Checked {index + 1}/{len(cases)} expressions", flush=True)
    terminals = [
        terminal(["R", "--quiet", "--no-readline", "--vanilla"]),
        terminal([str(snapshots[0]), "--quiet", "--no-save"]),
    ]

artifact = Path("target/options-parity.json")
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    reference_default_packages="NULL (base-only option probes)",
                                    source_sha256=source_hashes,
                                    binary_sha256=hashes, cases=results, terminals=terminals),
                               indent=2) + "\n")
failed = [row for row in results if not row["passed"] and not row["limit_observed"]]
failed += [row for row in terminals if not row["passed"]]
limits = [row for row in results if row["known_limit"] and row["limit_observed"]]
for row in failed:
    print(json.dumps(row))
print(f"{len(cases)} expressions, {len(results)} batch comparisons, {len(terminals)} terminal checks; {len(failed)} failures")
print(f"{len(limits)} known print/echo limits observed, not counted as passing")
print("Values/stdout/status compared. Warning presence checked; diagnostic wording and condition objects not compared.")
if failed:
    raise SystemExit(1)
