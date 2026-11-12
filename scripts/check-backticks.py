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
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
locale = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(Sys.getlocale("LC_CTYPE"))'],
                        env=environment, capture_output=True, text=True, check=True)
names = [(name, name) for name in ["my var", "if", "else", "for", "in", "while", "repeat",
         "function", "next", "break", "TRUE", "FALSE", "NULL", "NA", "NA_integer_",
         "NA_real_", "NA_complex_", "NA_character_", "Inf", "NaN", "T", "F", "_", "r", "R",
         "123", "+", "%custom%", "α😀", "#;()[]{}", "a\nb", "a\r\nb"]]
names += [(r"a\`b\\c", "a`b\\c"), (r"\a\b\f\n\r\t\v", "\x07\x08\x0c\n\r\t\x0b"),
          (r'''\'\"\ ''', "'\" "), (r"\1\12\1234\x41b", "\x01\nS4Ab"),
          (r"\303\251", "é"), (r"\xC3\xA9", "é"), (r"\xE2\x80\xAE", "\u202e"),
          (r"\\u0061", r"\u0061"), ("a\\\nb", "a\nb")]
cases = []
for i, (body, name) in enumerate(names):
    source = f'`{body}`<-7;cat(`{body}`,"\\n",sep="")\nx<-list(`{body}`=11);cat(names(x),":",x$`{body}`,"\\n",sep="")\n'
    cases.append(dict(label=f"name-{i}", source=source,
                      expected="7\n" + name.replace("\r\n", "\n") + ":11\n", status=0))

valid = [
    ("reference-snippet", '`my var`<-3;`my var`+1\n', "[1] 4\n"),
    ("parameters", 'f<-function(`if`,`a b`=2)`if`+`a b`;cat(f(`if`=1,`a b`=4))\n', "5"),
    ("omitted", 'cat((function(`a b`=3)`a b`)(`a b`=))\n', "3"),
    ("missing", 'cat((function(`a b`=3)missing(`a b`))(`a b`=))\n', "TRUE"),
    ("loop", 'for(`loop var` in 1:3)NULL;cat(`loop var`)\n', "3"),
    ("member-replace", 'x<-list(`a b`=1);x$`a b`<-12;cat(x$`a b`)\n', "12"),
    ("subscript-replace", '`a b`<-c(1,2);`a b`[2]<-4;cat(`a b`,sep=",")\n', "1,4"),
    ("super-assignment", '`a b`<-1;f<-function(){`a b`<<-4};f();cat(`a b`)\n', "4"),
    ("right-assignment", '1->`a b`;2->>`a b`;cat(`a b`)\n', "2"),
    ("quoted-function", '`f name`<-function(`x y`)`x y`;cat(`f name`(4))\n', "4"),
    ("quoted-keyword-function", '`if`<-function(x)x;cat(`if`(4))\n', "4"),
    ("quoted-builtin", 'cat(`sum`(1:4))\n', "10"),
    ("forward-dots", 'cat((function(...)`sum`(`...`))(1,2))\n', "3"),
    ("dots-tag", 'x<-list(`...`=11);cat(names(x),x$`...`,sep=":")\n', "...:11"),
]
cases += [dict(label=label, source=source, expected=expected, status=0)
          for label, source, expected in valid]
invalid = [f"`{body}`" for body in ["", r"\u0061", r"\u{61}", r"\U00000061", r"\u", r"\U",
           r"\z", r"\8", r"\x", r"\0", r"\x00", r"\400", r"\777"]]
invalid += [f"`a{chr(point)}b`" for point in [*range(0x202a, 0x202f), *range(0x2066, 0x206a)]]
invalid += ["`", "`unfinished", "`trailing\\", "`\\x41", "`\\u", "`\\U", "`a\n",
            "if(TRUE) 1 `else` 2", "for(x `in` 1:3) x", "...", "`...`",
            "`...`<-7;`...`", "(function(...)`...`)(1)"]
cases += [dict(label=f"invalid-{i}", source=f'cat("BEFORE\\n")\n{source}\n',
               expected="BEFORE\n", status=1) for i, source in enumerate(invalid)]
for label, body in [("ascii", ("a" * 999 + "\n") * 10),
                    ("unicode", ("é" * 499 + "\n") * 10 + "é" * 5),
                    ("escaped", (r"\x61" * 999 + "\n") * 10)]:
    for extra in ["", "x"]:
        name = body + extra
        status = 1 if extra else 0
        cases.append(dict(label=f"limit-{label}-{status}",
                          source=f'cat("BEFORE\\n")\n`{name}`<-7;cat(`{name}`)\n',
                          expected="BEFORE\n" if status else "BEFORE\n7", status=status,
                          modes=["stdin", "rscript-file"]))
for label, source, expected in [
    ("operator-call", 'cat(`+`(1,2))', "BEFORE\n3"),
    ("special-call", 'cat(`if`(TRUE,1,2))', "BEFORE\n1"),
    ("byte-name", r'`\xff`<-7;cat(`\xff`)', "BEFORE\n7"),
]:
    cases.append(dict(label=label, source='cat("BEFORE\\n")\n' + source + "\n",
                      expected=expected, status=0, known_limit=True))


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command, data):
    result = subprocess.run(command, input=data, env=environment, capture_output=True, timeout=30)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout), stderr=recorded(result.stderr))


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-backticks-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "quoted names.R"
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
            passed = r.returncode == u.returncode == case["status"] and r.stdout == u.stdout == case["expected"].encode()
            passed = passed and (bool(r.stderr) and bool(u.stderr) if case["status"] == 1
                                 else not r.stderr and not u.stderr)
            observed_limit = (case.get("known_limit", False) and r.returncode == 0 and not r.stderr
                              and r.stdout == case["expected"].encode() and u.returncode == 1
                              and u.stdout == b"BEFORE\n" and bool(u.stderr))
            results.append(dict(mode=mode, case=case, r=rr, rust=uu, passed=passed, observed_limit=observed_limit))
            count += 1
        print(f"Checked {mode}: {count} cases", flush=True)

Path("target/backtick-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_lc_ctype=locale.stdout,
    reference_default_packages="NULL", source_sha256=sources, binary_sha256=hashes, cases=results), indent=2) + "\n")
failed = [row for row in results if not row["passed"] and not row["observed_limit"]]
for row in failed[:12]:
    print(json.dumps(dict(mode=row["mode"], label=row["case"]["label"], r=row["r"], rust=row["rust"])))
print(f"{sum(row['passed'] for row in results)} passing backtick comparisons; "
      f"{sum(row['observed_limit'] for row in results)} known limits; {len(failed)} unexpected differences")
print("Exact stdout/status checked; successful stderr must be empty; error wording not compared.")
if failed:
    raise SystemExit(1)
