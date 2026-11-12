#!/usr/bin/env python3
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
environment = os.environ.copy()
environment["R_DEFAULT_PACKAGES"] = "NULL"
locale = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True)
programs = [
    ("value", "options(echo=TRUE)\n1", 0),
    ("continuation", "options(echo=TRUE)\n1+\n2", 0),
    ("comment", "options(echo=TRUE)\n# comment", 0),
    ("blank", "options(echo=TRUE)\n", 0),
    ("echo-off", "options(echo=TRUE)\n1\noptions(echo=FALSE)", 0),
    ("echo-na", "options(echo=NA)\n1", 0),
    ("prompt", 'options(echo=TRUE,prompt="α? ",continue="... ")\n(\n1+\n2)', 0),
    ("shebang", "#!/usr/bin/env Rscript\noptions(echo=TRUE)\n1", 0),
    ("unicode", "options(echo=TRUE)\ncat('α')", 0),
    ("literal-cr", 'options(echo=TRUE)\ncat("a\rb")', 0),
    ("last", ".Last<-function()cat('LAST')\noptions(echo=TRUE)\n1", 0),
    ("quit", "options(echo=TRUE)\nprint(1);q('no',status=3)\n@", 3),
    ("error", "options(echo=TRUE)\nprint(1);missingName\n9", 1),
    ("incomplete", "options(echo=TRUE)\n1+", 1),
]
cases = [dict(label=f"{label}-suffix-{len(suffix)}", source=source + suffix, status=status)
         for label, source, status in programs for suffix in ["", "\n", "\n\n"]]
cases += [dict(label=f"crlf-{label}", source=(source + "\n").replace("\n", "\r\n"), status=status)
          for label, source, status in programs[:7]]
cases += [
    dict(label="empty", source="", status=0),
    dict(label="bare-cr-eof", source="options(echo=TRUE)\n1\r", status=1),
    dict(label="bare-cr-between", source="options(echo=TRUE)\r1\r", status=1),
    dict(label="comment-cr-eof", source="options(echo=TRUE)\n# comment\r", status=0),
    dict(label="escape-cr-eof", source="options(echo=TRUE)\ncat('a\\\r", status=1),
    dict(label="literal-crlf", source='options(echo=TRUE)\ncat("a\r\nb")\n', status=0),
    dict(label="metadata", source="writeLines(commandArgs(TRUE));cat(interactive())\n", status=0),
]
blanks = [0x20, 0x09, 0x0c, 0x1680, *range(0x2000, 0x2007),
          *range(0x2008, 0x200b), 0x205f, 0x3000]
nonblanks = [0x0d, 0x0b, 0x85, 0xa0, 0x2007, 0x2028, 0x2029, 0x202f]
for point in blanks + nonblanks:
    status = 0 if point in blanks else 1
    for only in [False, True]:
        cases.append(dict(label=f"whitespace-{point:x}-only-{only}",
                          source=chr(point) if only else "1" + chr(point) + "+2\n", status=status))


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command, source):
    result = subprocess.run(command, input=source, env=environment,
                            capture_output=True, timeout=30)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout),
                        stderr=recorded(result.stderr))


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-file-input-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "file with spaces.R"
    trailing = ["literal", "--args", ""]
    for mode in ["file", "rscript-file", "stdin"]:
        for case in cases:
            source = case["source"].encode()
            if mode == "rscript-file":
                path.write_bytes(source)
                reference = ["Rscript", "--vanilla", str(path), *trailing]
                native = [str(snapshots[1]), "--vanilla", str(path), *trailing]
                data = b""
            else:
                reference = ["R", "--vanilla", "--no-echo"]
                native = [str(snapshots[0]), "--vanilla", "--no-echo"]
                data = source
                if mode == "file":
                    path.write_bytes(source)
                    reference += ["-f", str(path)]
                    native += ["-f", str(path)]
                    data = b""
                reference += ["--args", *trailing]
                native += ["--args", *trailing]
            r, rr = run(reference, data)
            u, uu = run(native, data)
            passed = r.returncode == u.returncode == case["status"] and r.stdout == u.stdout
            passed = passed and (bool(r.stderr) and bool(u.stderr) if case["status"] == 1
                                 else not r.stderr and not u.stderr)
            results.append(dict(mode=mode, case=case, r=rr, rust=uu, passed=passed))
        print(f"Checked {mode}: {len(cases)} cases", flush=True)

Path("target/file-input-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_lc_ctype=locale.stdout,
    reference_default_packages="NULL", source_sha256=sources,
    binary_sha256=hashes, cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"]]
for row in failed[:12]:
    print(json.dumps(dict(mode=row["mode"], label=row["case"]["label"], source=row["case"]["source"],
                         r=row["r"], rust=row["rust"])))
print(f"{len(results)} file/stdin comparisons; {len(failed)} differences")
print("Exact stdout/status checked; successful stderr must be empty; error wording not compared.")
if failed:
    raise SystemExit(1)
