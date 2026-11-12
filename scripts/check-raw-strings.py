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
environment = os.environ.copy()
environment["R_DEFAULT_PACKAGES"] = "NULL"
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
locale = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True)
literals = []
for prefix in ["r", "R"]:
    for quote in ["'", '"']:
        for opener, closer in [("(", ")"), ("[", "]"), ("{", "}"), ("|", "|")]:
            for dashes in ["", "-", "---"]:
                for value in ["", r"C:\path\n\z\u0041", "both ' and \" quotes",
                              "α😀\n# ; ()[]{}||", "a\r\nb", r"\0\u202e\xFF\c",
                              "a\t\x07\x08\x0c\x0bb"]:
                    literals.append((f"{prefix}{quote}{dashes}{opener}{value}{closer}{dashes}{quote}", value))
            other = '"' if quote == "'" else "'"
            value = f"{closer}-{quote}X{closer}---{quote}Y{closer}--{other}"
            literals.append((f"{prefix}{quote}--{opener}{value}{closer}--{quote}", value))

cases = []
for start in range(0, len(literals), 16):
    batch = literals[start:start + 16]
    source = "".join(f's<-{literal};cat("{start + i}:",nchar(s),":",s,"\\n",sep="")\n'
                     for i, (literal, _) in enumerate(batch))
    values = [value.replace("\r\n", "\n") for _, value in batch]
    expected = "".join(f"{start + i}:{len(value)}:{value}\n" for i, value in enumerate(values))
    cases.append(dict(label=f"values-{start}", source=source, expected=expected, status=0))

invalid = ['r""', 'r"x"', 'r"-x"', 'R"', 'r"--"', 'r"<text>"', "r'wrong'",
           'r"(', 'r"-[text', "R'--{text}--\"", 'r"|text', 'r"-(text)"']
invalid += [f'r"(a{chr(point)}b)"' for point in [*range(0x202a, 0x202f), *range(0x2066, 0x206a)]]
for i, literal in enumerate(invalid):
    cases.append(dict(label=f"invalid-{i}", source=f'cat("BEFORE\\n")\n{literal}\ncat("AFTER")\n',
                      expected="BEFORE\n", status=1))
cases += [
    dict(label="automatic-print", source='r"(C:\\path)"\n', expected='[1] "C:\\\\path"\n', status=0),
    dict(label="identifiers-and-closure", source='r<-1;R<-2;rate<-r+R\nf<-function(x)paste0(x,r"(\\n)")\ncat(rate, f(r"[a]"),sep="|")\n',
         expected="3|a\\n", status=0),
    dict(label="nested-multiline", source='cat(c(r"(a\n# not a comment\n)", R\'--[b]--\'),sep="|")\n',
         expected="a\n# not a comment\n|b", status=0),
]
for prefix in ["r", "R"]:
    value = "α\n" * 5100 + r"\u0041"
    cases.append(dict(label=f"long-{prefix}", source=f'cat({prefix}"---({value})---")\n',
                      expected=value, status=0, modes=["stdin", "rscript-file"]))


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command, data):
    result = subprocess.run(command, input=data, env=environment, capture_output=True, timeout=30)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout),
                        stderr=recorded(result.stderr))


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-raw-string-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "raw strings.R"
    for mode in ["stdin", "rscript-file", "expression", "rscript-expression"]:
        count = 0
        for case in cases:
            if mode not in case.get("modes", ["stdin", "rscript-file", "expression", "rscript-expression"]):
                continue
            source = case["source"]
            reference = ["R", "--vanilla", "--no-echo"]
            native = [str(snapshots[0]), "--vanilla", "--no-echo"]
            data = source.encode()
            if mode == "rscript-file":
                path.write_bytes(data)
                reference = ["Rscript", "--vanilla", str(path)]
                native = [str(snapshots[1]), "--vanilla", str(path)]
                data = b""
            elif mode in ["expression", "rscript-expression"]:
                expression = source.replace("\\", "\\\\")
                assert len(expression.encode()) + expression.count(" ") + expression.count("\t") + expression.count("\n") + 2 <= 10000
                if mode == "rscript-expression":
                    reference = ["Rscript", "--vanilla"]
                    native = [str(snapshots[1]), "--vanilla"]
                reference += ["-e", expression]
                native += ["-e", expression]
                data = b""
            r, rr = run(reference, data)
            u, uu = run(native, data)
            expected = case["expected"].encode()
            passed = r.returncode == u.returncode == case["status"] and r.stdout == u.stdout == expected
            passed = passed and (bool(r.stderr) and bool(u.stderr) if case["status"] == 1
                                 else not r.stderr and not u.stderr)
            results.append(dict(mode=mode, case=case, r=rr, rust=uu, passed=passed))
            count += 1
        print(f"Checked {mode}: {count} cases", flush=True)

Path("target/raw-string-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_lc_ctype=locale.stdout,
    reference_default_packages="NULL", source_sha256=sources, binary_sha256=hashes,
    literal_values=len(literals), cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"]]
for row in failed[:12]:
    print(json.dumps(dict(mode=row["mode"], label=row["case"]["label"],
                         r=row["r"], rust=row["rust"])))
print(f"{len(results)} raw-string comparisons; {len(failed)} differences")
print("Exact values/stdout/status checked; successful stderr must be empty; error wording not compared.")
if failed:
    raise SystemExit(1)
