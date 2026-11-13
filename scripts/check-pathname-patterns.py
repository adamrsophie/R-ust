#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
from __future__ import annotations

import base64
import hashlib
import itertools
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import TypedDict


class Comparison(TypedDict):
    pattern: str
    shell_fields: list[str]
    status: int
    stdout: str
    stderr: str
    expected_stdout: str
    passed: bool


def encoded(value: bytes) -> str:
    return base64.b64encode(value).decode("ascii")


def main() -> None:
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
    version = subprocess.run(["/bin/sh", "--version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    if "version 3.2.57" not in version:
        raise SystemExit("Independent macOS Bash 3.2.57 /bin/sh is required")
    sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
               for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                            Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
    environment = dict(os.environ, LC_ALL="C", R_DEFAULT_PACKAGES="NULL")
    rows: list[Comparison] = []
    with tempfile.TemporaryDirectory(prefix="r-ust bracket matrix ") as directory:
        root = Path(directory)
        native = root / binary.name
        _ = shutil.copy2(binary, native)
        binary_hash = hashlib.sha256(native.read_bytes()).hexdigest()
        names = ["p", *["p" + chr(value) for value in range(1, 128) if value != 47],
                 *["p" + suffix for suffix in ["[a]", "[b]", "[g]", "[:]", "a]", "b]", "g]", ":]",
                                                "ab", "aa", "1a", "a*b", "α"]]]
        for name in names:
            _ = (root / name).write_bytes(b"")
        classes = ["[ab]", "[!ab]", "[^ab]", "[]a]", "[-a]", "[a-]", "[z-a]", "[az-a]", "[z-a0]",
                   "[!z-a]", "[[=a=]]", "[[=b=]]", "[[=]=]]", "[!a[=b=]]", "[a[=b=]]", "[[=a=]b]",
                   "[a[:bogus:]]", "[[:bogus:]a]", "[![:bogus:]]", "[!a[:bogus:]]",
                   "[[:alpha:][:digit:]]", "[a-[.z.]]", "[[.a.]-z]", "[[.bogus.]]",
                   "[[.NUL.]-a]", "[[.slash.]-z]", "[[.solidus.]-z]",
                   "[", "[a", "[!", r"[\\", r"[a\\]", r"[\\]a]"]
        classes.extend(f"[[:{name}:]]" for name in ["alnum", "alpha", "ascii", "blank", "cntrl", "digit",
                                                       "graph", "lower", "print", "punct", "space", "upper",
                                                       "word", "xdigit", "bogus"])
        aliases = ["NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "alert", "BS", "backspace", "HT", "tab",
                   "LF", "newline", "VT", "vertical-tab", "FF", "form-feed", "CR", "carriage-return", "SO", "SI",
                   "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC",
                   "IS4", "FS", "IS3", "GS", "IS2", "RS", "IS1", "US", "space", "exclamation-mark",
                   "quotation-mark", "number-sign", "dollar-sign", "percent-sign", "ampersand", "apostrophe",
                   "left-parenthesis", "right-parenthesis", "asterisk", "plus-sign", "comma", "hyphen",
                   "hyphen-minus", "minus", "dash", "period", "full-stop", "slash", "solidus", "zero", "one",
                   "two", "three", "four", "five", "six", "seven", "eight", "nine", "colon", "semicolon",
                   "less-than-sign", "equals-sign", "greater-than-sign", "question-mark", "commercial-at",
                   "left-square-bracket", "backslash", "reverse-solidus", "right-square-bracket", "circumflex",
                   "circumflex-accent", "underscore", "grave-accent", "left-brace", "left-curly-bracket",
                   "vertical-line", "right-brace", "right-curly-bracket", "tilde", "DEL"]
        patterns = ["p" + prefix + value + suffix
                    for prefix, value, suffix in itertools.product(["", "*", "?"], classes, ["", "*", "?"])]
        patterns.extend("p" + prefix + f"[[.{name}.]]" for prefix, name in itertools.product(["", "*", "?"], aliases))
        for index, pattern in enumerate(patterns, 1):
            shell = subprocess.run(["/bin/sh", "-c", 'printf "%s\\000" $1', "sh", pattern],
                                   cwd=root, env=environment, capture_output=True, check=True, timeout=10)
            fields = shell.stdout.split(b"\0")[:-1]
            expected = b"".join(b"ARGUMENT '" + field + b"' __ignored__\n\n" for field in fields) + b"BODY"
            result = subprocess.run([str(native), "--vanilla", "--no-echo", pattern, "-e", "cat('BODY')"],
                                    input=b"", cwd=root, env=environment, capture_output=True, timeout=10, check=False)
            passed = (result.returncode, result.stdout, result.stderr) == (0, expected, b"")
            rows.append(Comparison(pattern=pattern, shell_fields=[encoded(field) for field in fields],
                                   status=result.returncode, stdout=encoded(result.stdout), stderr=encoded(result.stderr),
                                   expected_stdout=encoded(expected), passed=passed))
            if index % 100 == 0:
                print(f"Checked {index}/{len(patterns)} patterns", flush=True)
    assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in sources.items()), "Sources changed during comparison"
    _ = Path("target/pathname-pattern-parity.json").write_text(json.dumps({
        "reference_version": version, "source_sha256": sources, "binary_sha256": {"r_ust_cli": binary_hash},
        "cases": rows}, indent=2) + "\n")
    failed = [row for row in rows if not row["passed"]]
    for row in failed:
        print(f"FAIL {row['pattern']!r}: status {row['status']}")
    print(f"{len(rows)} shell-pattern comparisons; {len(failed)} differences; exact native stdout, stderr and status")
    print("C/POSIX byte matching only. Unterminated collating/equivalence forms with unsafe reference reads are not run.")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
