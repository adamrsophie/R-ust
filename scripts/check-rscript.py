"""Compare the native Rust Rscript frontend with installed Rscript 4.6.0."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/Rscript").resolve()
reference = shutil.which("Rscript")
if not reference or Path(reference).resolve() == binary:
    raise SystemExit("An independent Rscript 4.6.0 reference is required on PATH")
version = subprocess.run([reference, "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("Reference must be Rscript 4.6.0: " + version.stdout + version.stderr)

cases = []
for code in [
    "1+1", "sum(1:5)", "interactive()", "length(commandArgs(TRUE))",
    "writeLines(commandArgs()[-1])", "x <- 1\n\twriteLines(commandArgs()[-1])",
    'writeLines("literal~+~text")', 'writeLines("~n~+~ ~+~n~ α")',
    '.Last<-function()print(8);1+1', '.Last<-function()print(8);q("no",status=3);@',
]:
    cases.append(("valid", ["--vanilla", "-e", code], ""))
for trailing in [[], ["--args", "one", ""], ["--help", "-e", "print(99)"],
                 ["α", "two words", "--flag", "", "~+~", "~n~", "a\nb"]]:
    cases.append(("valid", ["--vanilla", "-e", "writeLines(commandArgs(TRUE))", *trailing], ""))
for option in ["--quiet", "--silent", "--no-save", "--no-echo", "--slave", "--no-restore",
               "--no-restore-data", "--no-restore-history", "--no-environ", "--no-site-file", "--no-init-file"]:
    cases.append(("valid", ["--vanilla", option, "-e", "writeLines(commandArgs()[-1])"], ""))
cases.extend([
    ("valid", ["--vanilla --quiet", "-e", "interactive()"], ""),
    ("valid", ["--vanilla", "-e", "x<-2 # comment", "-e", "x+3"], ""),
    ("valid", ["--vanilla", "-e", "sum(", "-e", "1,2)"], ""),
    ("valid", ["--vanilla", "-", "stdin-arg"], "x<-2\nsum(\nx,3)\nwriteLines(commandArgs(TRUE))\ninteractive()\n"),
    ("verbose", ["--vanilla", "--verbose", "-e", "1+1"], ""),
    ("usage", [], ""), ("help", ["--help"], ""), ("version", ["--version"], ""),
])
for args in [["--vanilla"], ["--vanilla", "-e"], ["--vanilla", "-e", ""],
             ["--vanilla", "-e", "-1"],
             ["--vanilla", "-e", " \t\n"], ["--vanilla", "-e", "  -1"],
             ["--vanilla", "-e", "print(1);@"], ["--vanilla", "-e", "print(1);missing_value"]]:
    cases.append(("error", args, ""))
cases.append(("input-error", ["--vanilla", "missing.R"], ""))
cases.append(("input-error", ["--vanilla", ""], ""))
for option in ["--save", "--restore", "--default-packages=NULL", "--default-packages=stats"]:
    cases.append(("unsupported", ["--vanilla", option, "-e", "print(9)"], ""))

results = []
with tempfile.TemporaryDirectory(prefix="r-ust-rscript-parity-") as directory:
    snapshot = Path(directory) / "Rscript"
    shutil.copy2(binary, snapshot)
    binary_sha256 = hashlib.sha256(snapshot.read_bytes()).hexdigest()
    script = Path(directory) / "script with spaces.R"
    script.write_text("#!/usr/bin/env Rscript\nwriteLines(commandArgs()[-1]);interactive()\n")
    cases.append(("valid", ["--vanilla", str(script), "--help", "-e", "print(99)", ""], ""))
    for kind, args, source in cases:
        r = subprocess.run([reference, *args], input=source, cwd=directory,
                           capture_output=True, text=True, timeout=20)
        u = subprocess.run([str(snapshot), *args], input=source, cwd=directory,
                           capture_output=True, text=True, timeout=20)
        if kind == "unsupported":
            passed = r.returncode == 0 and u.returncode != 0 and "not implemented" in u.stderr
        elif kind in ("usage", "help"):
            passed = (r.returncode == u.returncode and r.stdout.startswith("Usage: Rscript")
                      and u.stdout.startswith("Usage: Rscript") and not r.stderr and not u.stderr)
        elif kind == "input-error":
            passed = (r.returncode == u.returncode == 2 and not r.stderr and not u.stderr
                      and r.stdout.startswith("Fatal error: cannot open file")
                      and u.stdout.startswith("Fatal error: cannot open file"))
        elif kind == "version":
            passed = r.returncode == u.returncode == 0 and "Rscript (R-ust)" in u.stdout and not u.stderr
        else:
            passed = r.returncode == u.returncode and r.stdout == u.stdout
            if kind == "valid":
                passed = passed and not r.stderr and not u.stderr
            elif kind == "error":
                passed = passed and r.returncode != 0
            elif kind == "verbose":
                passed = passed and "running" in r.stderr and "R-ust Rscript arguments:" in u.stderr
        results.append(dict(kind=kind, args=args, stdin=source, passed=passed,
                            r_status=r.returncode, rust_status=u.returncode,
                            r_stdout=r.stdout, rust_stdout=u.stdout,
                            r_stderr=r.stderr, rust_stderr=u.stderr))
failed = [row for row in results if not row["passed"]]
artifact = Path("target/rscript-parity.json")
artifact.parent.mkdir(parents=True, exist_ok=True)
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    binary_sha256=binary_sha256,
                                    cases=results), indent=2) + "\n")
for row in failed:
    print(json.dumps(row))
unsupported = sum(row["kind"] == "unsupported" for row in results)
print(f"{len(results)} Rscript probes; {unsupported} unsupported (not passing); {len(failed)} mismatches")
print("Error status and prior stdout checked; diagnostic, help, version, and verbose wording not compared.")
if failed:
    raise SystemExit(1)
