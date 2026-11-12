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


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command, source=None):
    result = subprocess.run(command, input=source, capture_output=True, env=environment, timeout=20)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout),
                        stderr=recorded(result.stderr))


valid = [(r"\a\b\f\n\r\t\v", "\x07\x08\x0c\n\r\t\x0b"),
         (r'''\\\'\"\`\ ''', "\\'\"` "), ("a\\\nb", "a\nb"), ("a\nb", "a\nb"),
         (r"\1\12\1234\x41b\x4z", "\x01\nS4Ab\x04z"), ("", ""),
         (r"\uD83D\uDE00", "😀"), (r"\U0000d83d\U0000de00", "😀"),
         (r"é\u0061\U{1f600}", "éa😀"), (r"\u1Z\U1Z\u0061F", "\x01Z\x01ZaF")]
for style in ("octal", "hex"):
    body = "".join((rf"\{value:03o}" if style == "octal" else rf"\x{value:02x}") + "!"
                   for value in range(1, 128))
    valid.append((body, "".join(chr(value) + "!" for value in range(1, 128))))
for value in [1, 0x41, 0x7f, 0x80, 0xe9, 0x3b1, 0x7ff, 0x800, 0x202e, 0xd7ff,
              0xe000, 0xffff, 0x10000, 0x1f600, 0x10ffff]:
    for prefix, width in [("u", 4), ("U", 8)]:
        if prefix == "u" and value > 0xffff:
            continue
        valid += [("\\" + prefix + f"{value:0{width}x}", chr(value)),
                  ("\\" + prefix + "{" + f"{value:x}" + "}", chr(value))]
for text in ["é", "α", "😀", "aéα😀"]:
    for style in ("octal", "hex"):
        valid.append(("".join(rf"\{byte:03o}" if style == "octal" else rf"\x{byte:02x}"
                              for byte in text.encode()), text))

invalid = ["\\" + suffix for suffix in ["z", "e", "E", "c", "8", "9", "?", "/", "#", "!",
            "x", "u", "U", "xg", "ug", "Uq", "000", "x00", "u0000", "U00000000",
            "400", "401", "777", "U00110000", "Uffffffff", "u{00001}", "U{000000001}",
            "u{a", "u{}", "u0061\\x41", "101\\u0061", "u{61}\\1"]]
invalid += [chr(value) for value in [*range(0x202a, 0x202f), *range(0x2066, 0x206a)]]
cases = []
for quote in ["'", '"']:
    for body, value in valid:
        literal = quote + body + quote
        cases.append(dict(kind="valid", source=f"s<-{literal};cat(nchar(s),':',s,sep='')\n",
                          expected=(str(len(value)) + ":" + value).encode()))
    for body in invalid:
        cases.append(dict(kind="error", source=f"cat('before\\n')\ncat({quote}{body}{quote})\ncat('after\\n')\n"))
cases += [dict(kind="error", source=source) for source in ['cat("before\\n")\n"unfinished',
          'cat("before\\n")\n"trailing\\', 'cat("before\\n");"\\z"']]
cases.append(dict(kind="valid", source='q("no")\n"\\z"\n', expected=b""))
for length in [9998, 9999]:
    source = 'cat("before\\n")\ns<-"\\u0061' + "x" * (length - 1) + '";cat(nchar(s))\n'
    cases.append(dict(kind="valid" if length == 9998 else "error", source=source,
                      expected=b"before\n9998" if length == 9998 else None, expression_boundary=True))
for body, error in [(r"\xff", "non-UTF-8 byte strings"), (r"\377", "non-UTF-8 byte strings"),
                    (r"\uD800", "unpaired surrogate strings"), (r"\uDC00", "unpaired surrogate strings")]:
    cases.append(dict(kind="known-encoding-limit", source=f'cat("{body}")\n', diagnostic=error))

sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-strings-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    path = Path(directory) / "strings.R"
    for mode in ["stdin", "expression", "rscript-file", "rscript-expression"]:
        for case in cases:
            source = case["source"]
            data = None
            if mode == "stdin":
                reference, native = ["R", "--vanilla", "--no-echo"], [str(snapshots[0]), "--no-echo"]
                data = source.encode()
            elif mode == "rscript-file":
                path.write_text(source)
                reference, native = ["Rscript", "--vanilla", str(path)], [str(snapshots[1]), str(path)]
            else:
                escaped = source.replace("\\", "\\\\")
                reference = ["R" if mode == "expression" else "Rscript", "--vanilla", "--no-echo", "-e", escaped]
                native = [str(snapshots[0] if mode == "expression" else snapshots[1]), "--no-echo", "-e", escaped]
            r, rr = run(reference, data)
            u, uu = run(native, data)
            kind = case["kind"]
            if case.get("expression_boundary") and "expression" in mode:
                kind = "known-expression-buffer-limit"
                native_ok = (u.returncode == 0 and u.stdout == case["expected"] and not u.stderr
                             if case["kind"] == "valid" else
                             u.returncode == 1 and u.stdout == b"before\n"
                             and b"Unicode escapes is too long" in u.stderr)
                passed = (r.returncode == 0 and not r.stderr and native_ok
                          and r.stdout.startswith(b"WARNING: '-e ") and len(r.stdout) == 1152)
            elif kind == "valid":
                passed = (r.returncode == u.returncode == 0 and r.stdout == u.stdout == case["expected"]
                          and not r.stderr and not u.stderr)
            elif kind == "error":
                passed = (r.returncode == u.returncode == 1 and r.stdout == u.stdout == b"before\n"
                          and bool(r.stderr) and bool(u.stderr))
            else:
                passed = (r.returncode == 0 and u.returncode == 1 and not u.stdout
                          and case["diagnostic"] in uu["stderr"]["text"])
            row = dict(kind=kind, source=source, mode=mode, r=rr, rust=uu)
            row["limit_observed" if kind.startswith("known-") else "passed"] = passed
            results.append(row)
        print(f"Checked {mode}: {len(cases)} cases", flush=True)

artifact = Path("target/string-escape-parity.json")
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    reference_default_packages="NULL", source_sha256=sources,
                                    binary_sha256=hashes, cases=results), indent=2) + "\n")
failed = [row for row in results if not row.get("passed", row.get("limit_observed", False))]
for row in failed[:10]:
    print(json.dumps(dict(kind=row["kind"], mode=row["mode"], source=row["source"][:120],
                         source_bytes=len(row["source"].encode()), r_status=row["r"]["status"],
                         rust_status=row["rust"]["status"])))
limits = sum(row["kind"].startswith("known-") for row in results)
print(f"{len(results) - limits} comparisons; {limits} excluded input/encoding limits; {len(failed)} unexpected differences")
print("Valid cases compare exact values/streams/status. Errors compare prior stdout, status and diagnostic presence.")
if failed:
    raise SystemExit(1)
