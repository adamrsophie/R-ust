"""Compare quit and statement execution against installed R after building r_ust_cli."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
cases = [
    ("valid", 'q("no",status=3); @'),
    ("valid", 'print(1);q("no");"unfinished'),
    ("valid", 'cat("é; # 😀");q("no");@'),
    ("valid", '{if(TRUE)1\nelse 2}'),
    ("error", 'print(1);@'),
    ("error", 'print(1);sum(\n'),
    ("error", '{q("no",status=3);@}'),
    ("error", '{print(1);@}'),
    ("error", 'if(TRUE)1\nelse 2'),
    ("error", '{if(TRUE)print(1);else(2)}'),
    ("error", 'print(1);missing_value;@'),
    ("valid", "print(1); q(); print(2)"),
    ("valid", 'quit("no", 7); print(2)'),
    ("valid", 'f<-function(){for(i in 1:3)q("no",status=3);99};f();99'),
    ("valid", 'lapply("no",quit);99'),
    ("valid", '(function(x)x)(quit("no",status=5));99'),
    ("valid", 'quit("no",st=6,run=FALSE)'),
    ("valid", 'quit(status=4,"no")'),
    ("valid", 'quit(save="no",s=2)'),
    ("valid", 'quit("no",status=257)'),
    ("valid", 'quit("no",status=-1)'),
    ("valid", 'quit("no",status="7.8")'),
    ("valid", 'quit(c("no","yes"),status=c(4,8))'),
    ("valid", '.Last<-function()print(8);q("no")'),
    ("valid", '.Last<-function()print(8);q("no",runLast=FALSE)'),
    ("valid", '.Last<-function()q("no",4,FALSE);q("no",3)'),
    ("valid", '.Last<-1;q("no")'),
    ("valid", '.Last<-function()print(8)'),
    ("valid", 'f<-function(){.Last<-function()print(9);q("no")};f()'),
    ("valid", 'f<-function(){q("no",status=return(3));99};f()'),
    ("valid", 'for(i in 1:2)q(save=break);print(9)'),
    ("valid", 'f<-function(...)q(...);f("no",status=4)'),
    ("error", 'q(bogus=print(9))'),
    ("error", 'q("no",0,FALSE,print(9))'),
    ("error", 'q(s="no")'),
    ("error", 'q(save="no",sa=print(9))'),
    ("error", 'q(save="no",save="no")'),
    ("error", 'q("bad")'),
    ("error", 'q(1)'),
    ("error", 'q(NA_character_)'),
    ("error", '.Last<-function()missing_value;q("no")'),
    ("warning", 'q("no",status=NA_integer_)'),
    ("warning", '.Last<-function()print(8);q("no",runLast=NA)'),
    ("warning", 'q("ask")'),
    ("unsupported", 'q("yes")'),
]
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-quit-parity-") as directory:
    for kind, code in cases:
        r = subprocess.run(["Rscript", "--vanilla", "-e", code], cwd=directory,
                           capture_output=True, text=True, timeout=15)
        u = subprocess.run([str(binary), "-e", code], cwd=directory,
                           capture_output=True, text=True, timeout=15)
        if kind == "error":
            passed = r.returncode != 0 and r.returncode == u.returncode and r.stdout == u.stdout
        elif kind == "unsupported":
            passed = r.returncode == 0 and u.returncode != 0 and "not implemented" in u.stderr
        else:
            passed = r.returncode == u.returncode and r.stdout == u.stdout
            if kind == "valid":
                passed = passed and r.stderr == u.stderr
            else:
                # Check warning content; the condition system and diagnostic layout remain partial.
                fragment = ("invalid 'status'" if "status=" in code else
                            "invalid 'runLast'" if "runLast=" in code else 'save="ask"')
                passed = passed and fragment in r.stderr and fragment in u.stderr
        results.append(dict(kind=kind, code=code, passed=passed,
                            r_status=r.returncode, rust_status=u.returncode,
                            r_stdout=r.stdout, rust_stdout=u.stdout,
                            r_stderr=r.stderr, rust_stderr=u.stderr))
failed = [row for row in results if not row["passed"]]
artifact = Path("target/cli-quit-parity.json")
artifact.parent.mkdir(parents=True, exist_ok=True)
r_version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
artifact.write_text(json.dumps(dict(r_version=r_version.stdout + r_version.stderr,
                                    binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                                    cases=results), indent=2))
for row in failed:
    print(json.dumps(row))
unsupported = sum(row["kind"] == "unsupported" for row in results)
print(f"{len(results)} quit probes; {unsupported} unsupported (not passing); {len(failed)} mismatches")
print("Error wording is not compared; warning content is checked without diagnostic layout.")
if failed:
    raise SystemExit(1)
