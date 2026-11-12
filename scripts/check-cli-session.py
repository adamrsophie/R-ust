"""Compare CLI session metadata and expression sequencing with installed R."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
trailing = ["α", "two words", "--flag", "", "--args"]
cases = []
for argument in ["TRUE", "trailingOnly=TRUE", "tr=TRUE", "1", "-1", '\"TRUE\"', '\"T\"', "invisible(TRUE)"]:
    cases.append(("valid", ["-e", f"writeLines(commandArgs({argument}))", "--args", *trailing]))
for argument in ["FALSE", "0", '\"FALSE\"', "trailingOnly="]:
    cases.append(("valid", ["-e", f"writeLines(tail(commandArgs({argument}),5))", "--args", *trailing]))
for code in [
    "length(commandArgs(TRUE))", "interactive()", "withVisible(commandArgs(TRUE))$visible",
    "(function(...)writeLines(commandArgs(...)))(TRUE)",
    "writeLines(lapply(TRUE,commandArgs)[[1]])",
    "(function()commandArgs(return(7)))()",
    "if(\"TRUE\") 1 else 2", "if(0) 1 else 2",
    "(function(x=3)x)(x=)", "(function(x=3,y=4)x)(,4)",
    "(function(x=3,y=4)x)(,)", "(function(x=3,y=4)y)(1,)",
    "(function(x=3)missing(x))(x=)", "q(save=,status=0)",
]:
    cases.append(("valid", ["-e", code, "--args", *trailing]))
for code in [
    "commandArgs(NA)", "commandArgs(NULL)", "commandArgs(c(TRUE,FALSE))",
    "commandArgs(c(FALSE,NA))", "commandArgs(list(TRUE))", 'commandArgs(" TRUE ")',
    "commandArgs(bogus=print(9))", "commandArgs(TRUE,print(9))",
    "commandArgs(tr=print(9),trailingOnly=TRUE)",
    "commandArgs(trailingOnly=TRUE,trailingOnly=print(9))",
    "interactive(1)", "interactive(print(9))",
    "if(NULL) 1", "if(c(TRUE,FALSE)) 1", "while(c(FALSE,TRUE)) 1",
    "commandArgs(,)", "(function(x)x)(x=)", "sum(1,)",
]:
    cases.append(("error", ["-e", code]))
cases.extend([
    ("valid", ["-e", "x<-2 # comment", "-e", "x+3"]),
    ("valid", ["-e", "sum(", "-e", "1,2)"]),
    ("valid", ["-e", "print(1)", "-e", "q(\"no\")", "-e", "@"]),
])
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-session-") as directory:
    for kind, args in cases:
        r = subprocess.run(["R", "--vanilla", "--slave", *args], cwd=directory,
                           capture_output=True, text=True, timeout=15)
        u = subprocess.run([str(binary), "--slave", *args], cwd=directory,
                           capture_output=True, text=True, timeout=15)
        passed = r.returncode == u.returncode and r.stdout == u.stdout
        passed = passed and (r.returncode != 0 if kind == "error" else not r.stderr and not u.stderr)
        results.append(dict(kind=kind, args=args, passed=passed,
                            r_status=r.returncode, rust_status=u.returncode,
                            r_stdout=r.stdout, rust_stdout=u.stdout,
                            r_stderr=r.stderr, rust_stderr=u.stderr))
failed = [row for row in results if not row["passed"]]
artifact = Path("target/cli-session-parity.json")
artifact.parent.mkdir(parents=True, exist_ok=True)
r_version = subprocess.run(["R", "--version"], capture_output=True, text=True, check=True)
artifact.write_text(json.dumps(dict(r_version=r_version.stdout + r_version.stderr,
                                    binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                                    cases=results), indent=2) + "\n")
for row in failed:
    print(json.dumps(row))
print(f"{len(results)} session probes; {len(failed)} mismatches")
print("Error status and prior stdout checked; diagnostic wording not compared.")
if failed:
    raise SystemExit(1)
