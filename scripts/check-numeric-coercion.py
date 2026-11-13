#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run: python3 scripts/check-numeric-coercion.py [target/debug/r_ust_cli]."""
from __future__ import annotations

import base64
import hashlib
import json
import math
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, Literal, assert_never

Mode = Literal["stdin", "rscript-file", "expression", "rscript-expression"]
MODES: Final[tuple[Mode, ...]] = ("stdin", "rscript-file", "expression", "rscript-expression")
ROOT: Final = Path(__file__).resolve().parent.parent


@dataclass(frozen=True, slots=True)
class Case:
    label: str
    source: str
    expected: str | None = None
    status: int = 0
    diagnostic: bool = False


@dataclass(frozen=True, slots=True)
class Stream:
    text: str
    base64: str


@dataclass(frozen=True, slots=True)
class Run:
    command: list[str]
    status: int
    stdout: Stream
    stderr: Stream


@dataclass(frozen=True, slots=True)
class Invocation:
    command: list[str]
    data: bytes
    environment: dict[str, str]


@dataclass(frozen=True, slots=True)
class Comparison:
    profile: str
    locale: str
    mode: Mode
    case: Case
    reference: Run
    native: Run
    merged: list[Run]
    passed: bool
    exact_streams: bool
    exact_merged: bool | None


@dataclass(frozen=True, slots=True)
class Locale:
    profile: str
    environment: str
    reference: Run


def run(invocation: Invocation, *, merged: bool = False) -> Run:
    result = subprocess.run(invocation.command, input=invocation.data, env=invocation.environment,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE,
                            timeout=30, check=False)
    streams = [Stream(data.decode("utf-8", errors="replace"), base64.b64encode(data).decode("ascii"))
               for data in (result.stdout, result.stderr or b"")]
    return Run(invocation.command, result.returncode, streams[0], streams[1])


def commands(mode: Mode, executables: tuple[str, str], source: Path) -> tuple[list[str], bytes]:
    text = source.read_text()
    match mode:
        case "stdin":
            return [executables[0], "--vanilla", "--no-echo"], text.encode()
        case "rscript-file":
            return [executables[1], "--vanilla", str(source)], b""
        case "expression" | "rscript-expression":
            expression = text.replace("\\", "\\\\")
            encoded = expression.replace(" ", "~+~").replace("\t", "~t~").replace("\n", "~n~")
            assert len(encoded.encode()) + 2 <= 10000, source
            prefix = [executables[0], "--vanilla", "--no-echo"] if mode == "expression" else [executables[1], "--vanilla"]
            return [*prefix, "-e", expression], b""
        case _:
            assert_never(mode)


def canonical(text: str) -> str:
    """Exclude call contexts and error prefixes, retaining diagnostic text and order."""
    text = text.replace("Execution halted\n", "")
    text = re.sub(r"(?m)^Calls: [^\n]*\n", "", text)
    text = re.sub(r"(?m)^(?:Warning messages?:|In addition: Warning messages?:)\n", "", text)
    text = re.sub(r"(?m)^(?:Warning in [^\n]*? :|\d+: In [^\n]*? :|In [^\n]*? :)\s*", "", text)
    text = re.sub(r"(?m)^(?:Warning: |\d+: )", "", text)
    text = re.sub(r"(?m)^(?:Error in [^\n]*? :|Error: |domain error: |type error: |argument error: )\s*", "ERROR: ", text)
    text = re.sub(r"(?m)^(NAs introduced by coercion(?: to integer range)?|invalid 'status', 0 assumed|invalid 'runLast', FALSE assumed) \n", r"\1\n", text)
    return text


def fixture_cases(profile: str) -> list[Case]:
    """Use independent finite bits and missing tags, never printed decimal values."""
    rows = (ROOT / f"crates/r_ust_eval/tests/fixtures/numeric-coercion-{profile}-r-4.6.0.tsv").read_text().splitlines()
    assert rows[0].startswith("# R version 4.6.0 ") and "aarch64-apple-darwin" in rows[0] and rows[0].endswith("LDOUBLE 8"), rows[0]
    expected_locale = "C" if profile == "C" else "C.UTF-8"
    assert f"LC_CTYPE {expected_locale} LDOUBLE 8" in rows[0], rows[0]
    cases: list[Case] = []
    batch: list[str] = []
    row_count, selected_count = 0, 0
    for row in rows:
        if row.startswith("#"):
            continue
        fields = row.split("\t")
        assert len(fields) == 6, row
        identifier, encoded, real, real_warning, integer, integer_warning = fields
        row_count += 1
        assert int(identifier) == row_count, row
        assert real_warning in ("-", "G") and integer_warning in ("-", "G", "R"), row
        if not (int(identifier) <= 92 or int(identifier) >= 1990):
            continue
        selected_count += 1
        value = "NA_character_" if encoded == "NA" else json.dumps("" if encoded == "-" else bytes.fromhex(encoded).decode(), ensure_ascii=False)
        if len(value) > 8190:
            value = 'paste(rep(" ",9000),collapse="")' if encoded.startswith("20") else value
        expression = f"s<-{value};x<-as.double(s);y<-as.integer(s);"
        if real in ("NA", "NaN"):
            check = "is.na(x)&!is.nan(x)" if real == "NA" else "is.nan(x)"
        else:
            assert re.fullmatch("[0-9a-f]{16}", real), row
            number = struct.unpack(">d", bytes.fromhex(real))[0]
            literal = ("-Inf" if number < 0 else "Inf") if math.isinf(number) else number.hex()
            check = f"x=={literal}"
            if number == 0:
                check += "&1/x==" + ("-Inf" if real.startswith("8") else "Inf")
        integer_check = "is.na(y)" if integer == "NA" else f"y=={int(integer)}"
        expression += f'cat(typeof(x)=="double"&typeof(y)=="integer"&({check})&({integer_check}),"\\n",sep="");\n'
        if sum(map(len, batch)) + len(expression) > 2300 and batch:
            cases.append(Case(f"fixture-{profile}-{identifier}", "options(warn=-1)\n" + "".join(batch), "TRUE\n" * len(batch)))
            batch = []
        batch.append(expression)
    assert row_count == 2061 and selected_count == 164, (row_count, selected_count)
    if batch:
        cases.append(Case(f"fixture-{profile}-last", "options(warn=-1)\n" + "".join(batch), "TRUE\n" * len(batch)))
    return cases


def diagnostic_cases() -> list[Case]:
    cases: list[Case] = []
    for level in (-1, 0, 1, 2, 3):
        for label, value in (("atomic", 'c("2147483648","bad","2147483648")'),
                             ("list", 'list("2147483648","bad","2147483648")')):
            source = f'options(warn={level})\ncat("BEFORE\\n")\ncat(as.integer({value}))\ncat("AFTER\\n")\n'
            cases.append(Case(f"{label}-warn-{level}", source, status=int(level >= 2), diagnostic=True))
        cases.append(Case(f"prevalidation-{level}", f'options(warn={level})\nas.integer(list("bad",c(1,2)))\n', status=1, diagnostic=True))
    for level in (-1, 0, 1, 2):
        for label, argument in (("quit-blank-status", 'status=""'), ("quit-runLast-NA", "runLast=NA")):
            source = f'options(warn={level});.Last<-function()cat("LAST\\n");q("no",{argument})\n'
            cases.append(Case(f"{label}-{level}", source, status=int(level >= 2), diagnostic=True))
    for label, source, status in (
        ("options-tail", 'options(warn=2);options(digits=c("7","bad"));cat(getOption("digits"))', 0),
        ("option-promotion", 'options(warn=2);options(digits="bad");cat("AFTER\\n")', 1),
        ("quit-tail", 'options(warn=2);q("no",status=c("7","bad"))', 7),
        ("quit-last", '.Last<-function()cat("LAST\\n");q("no",status="0")', 0),
        ("last-coercion", 'options(warn=1);.Last<-function(){cat(as.integer("bad"));cat("LAST\\n")};q("no")', 0),
        ("last-promotion", 'options(warn=2);.Last<-function(){cat(as.integer("bad"));cat("LAST\\n")};q("no")', 1),
        ("quit-promotion", 'options(warn=2);.Last<-function()cat("LAST\\n");q("no",status="bad")', 1),
        ("empty-nested", 'cat(as.integer(list(as.integer(),list(),list(1))))', 0),
        ("empty-aliases", 'cat(typeof(as.numeric()),typeof(as.double()),typeof(as.integer()))', 0),
        ("matrix", 'cat(as.double(matrix(c("1","2"),nrow=1)))', 0),
    ):
        cases.append(Case(label, source + "\n", status=status, diagnostic=True))
    return cases


def main() -> int:
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT / "target/debug/r_ust_cli").resolve()
    environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
    version = run(Invocation(["Rscript", "--version"], b"", environment))
    assert "Rscript (R) version 4.6.0" in version.stdout.text + version.stderr.text, version
    paths = [*ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"), *ROOT.glob("crates/**/Cargo.toml"),
             ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__)]
    sources = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    reference_paths = [Path(shutil.which(name) or name).resolve() for name in ("R", "Rscript")]
    checkout = [run(Invocation(["git", *arguments], b"", environment)) for arguments in (["rev-parse", "HEAD"], ["status", "--short"], ["log", "-1", "--format=fuller"])]
    references = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in reference_paths}
    metadata_source = 'cat(R.version$arch,Sys.getlocale("LC_CTYPE"),l10n_info()[["UTF-8"]],sep="/")'
    results: list[Comparison] = []
    locales: list[Locale] = []
    with tempfile.TemporaryDirectory(prefix="r-ust-numeric-coercion-") as directory:
        snapshots = [Path(directory) / name for name in ("r_ust_cli", "Rscript")]
        for original, snapshot in zip((binary, binary.parent / "Rscript"), snapshots, strict=True):
            _ = shutil.copy2(original, snapshot)
        hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
        profiles = [("C", "C"), ("Utf8", "en_US.UTF-8")]
        if sys.platform == "darwin":
            profiles.append(("Utf8", "unset"))
        for profile, locale in profiles:
            child = {key: value for key, value in environment.items() if key not in ("LC_ALL", "LC_CTYPE", "LANG")}
            if locale != "unset":
                child.update(LC_ALL=locale, LC_CTYPE=locale, LANG=locale)
            metadata = run(Invocation(["Rscript", "--vanilla", "-e", metadata_source], b"", child))
            assert metadata.status == 0 and not metadata.stderr.text, metadata
            assert metadata.stdout.text.endswith("/" + ("FALSE" if profile == "C" else "TRUE")), metadata
            locales.append(Locale(profile, locale, metadata))
            for mode in MODES:
                for case in fixture_cases(profile) + diagnostic_cases():
                    source = Path(directory) / "numeric coercion.R"
                    _ = source.write_text(case.source)
                    executions: list[Run] = []
                    for executables in (("R", "Rscript"), (str(snapshots[0]), str(snapshots[1]))):
                        command, data = commands(mode, executables, source)
                        executions.append(run(Invocation(command, data, child)))
                    reference, native = executions
                    diagnostics_match = canonical(reference.stderr.text) == canonical(native.stderr.text)
                    passed = reference.status == native.status == case.status and reference.stdout == native.stdout and diagnostics_match
                    passed = passed and (case.expected is None or reference.stdout.text == case.expected)
                    merged: list[Run] = []
                    if case.diagnostic:
                        for value in executions:
                            command, data = commands(mode, (value.command[0], value.command[0]), source)
                            merged.append(run(Invocation(command, data, child), merged=True))
                        passed = passed and canonical(merged[0].stdout.text) == canonical(merged[1].stdout.text)
                    results.append(Comparison(profile, locale, mode, case, reference, native, merged, passed,
                                              reference.stdout == native.stdout and reference.stderr == native.stderr,
                                              merged[0].stdout == merged[1].stdout if merged else None))
                print(f"Checked {locale}/{mode}", flush=True)
    stable = all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in sources.items())
    report = ROOT / "target/numeric-coercion-parity.json"
    _ = report.write_text(json.dumps({"reference_version": asdict(version), "reference_sha256": references,
        "checkout": [asdict(value) for value in checkout], "source_sha256": sources, "binary_sha256": hashes, "source_stable": stable, "locales": [asdict(value) for value in locales], "cases": [asdict(value) for value in results],
        "raw_stream_differences": sum(not row.exact_streams for row in results),
        "raw_merged_differences": sum(row.exact_merged is False for row in results),
        "fixture_selection": "IDs 1..92 and 1990..2061; full TSV corpus is owned by evaluator tests",
        "exclusions": ["Diagnostic call contexts including Calls traces, error prefixes, Execution halted; raw streams retained.",
                    "One ASCII deferred-warning pad space on exact known diagnostic lines; condition-call formatting gap.",
                    "as.real has no R reference function; evaluator alias tests own that extension.",
                    "Callback sink failures are covered by evaluator tests, not this process driver.",
                    "ARM binary64 arithmetic, C/UTF-8 child environment profiles only; no S3 dispatch claim."]}, indent=2) + "\n")
    failures = [row for row in results if not row.passed]
    for row in failures[:20]:
        print(json.dumps({"locale": row.locale, "mode": row.mode, "case": row.case.label}))
    print(f"{len(results) - len(failures)}/{len(results)} comparisons passed; source manifest stable: {stable}")
    return int(bool(failures) or not stable)


if __name__ == "__main__":
    raise SystemExit(main())
