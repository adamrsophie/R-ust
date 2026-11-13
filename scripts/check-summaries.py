#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# Run: uv run scripts/check-summaries.py [target/debug/r_ust_cli]
# Or: python3 scripts/check-summaries.py [target/debug/r_ust_cli]
"""Check representative summary contracts through four executable transports."""
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
FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/summary-r-4.6.0.tsv"
SELECTED: Final = (5, 13, 70, 1157, 1161, 1163, 1165, 1167, 1169, 1171, 1237, 1239, 1250,
                   1254, 1273, 1278, 1279, 1280, 1281, 1283, 1304, 1306, 1318, 1320,
                   1330, 1345, 1346, 2194, 2514, 2520, 2524, 2598, 2602, 2639,
                   2652, 2671, 2673, 2693, 2702)


@dataclass(frozen=True, slots=True)
class Case:
    label: str
    source: str
    stdout: str
    status: int = 0
    diagnostic: str = ""


@dataclass(frozen=True, slots=True)
class Run:
    command: tuple[str, ...]
    status: int
    stdout: str
    stderr: str
    raw_base64: tuple[str, str]


@dataclass(frozen=True, slots=True)
class Invocation:
    command: tuple[str, ...]
    data: bytes
    environment: dict[str, str]


@dataclass(frozen=True, slots=True)
class Comparison:
    mode: Mode
    case: Case
    reference: Run
    native: Run
    reference_merged: Run
    native_merged: Run
    passed: bool
    exact_streams: bool
    exact_merged: bool


def run(invocation: Invocation, *, merged: bool = False) -> Run:
    result = subprocess.run(invocation.command, input=invocation.data, env=invocation.environment,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE,
                            timeout=30, check=False)
    stdout, stderr = result.stdout, result.stderr or b""
    return Run(invocation.command, result.returncode, stdout.decode(), stderr.decode(),
               (base64.b64encode(stdout).decode(), base64.b64encode(stderr).decode()))


def commands(mode: Mode, executables: tuple[str, str], source: Path) -> tuple[tuple[str, ...], bytes]:
    match mode:
        case "stdin":
            return (executables[0], "--vanilla", "--no-echo"), source.read_bytes()
        case "rscript-file":
            return (executables[1], "--vanilla", str(source)), b""
        case "expression" | "rscript-expression":
            prefix = (executables[0], "--vanilla", "--no-echo") if mode == "expression" else (executables[1], "--vanilla")
            return (*prefix, "-e", source.read_text().replace("\\", "\\\\")), b""
        case _:
            assert_never(mode)


def diagnostic_layout(text: str) -> str:
    """Remove only condition layout, preserving messages, multiplicity and order."""
    text = re.sub(r"(?m)^Execution halted\n", "", text)
    text = re.sub(r"(?m)^Calls: [^\n]*\n", "", text)
    text = re.sub(r"(?m)^(?:Error in [^\n]*? :[ \t]*(?:\n  )?|Error: |(?:domain|type|argument) error: )", "ERROR: ", text)
    return re.sub(r"(?m)^ERROR: ‘(sum|prod)’ not meaningful for factors$", r"ERROR: '\1' not meaningful for factors", text)


def comparator_self_check() -> None:
    expected = "BEFORE\nWarning: first\nWarning: second\nERROR: failure\n"
    original = "BEFORE\nWarning: first\nWarning: second\nError in sum(1) : failure\nExecution halted\n"
    assert diagnostic_layout(original) == expected
    for changed in (original.replace("first", "changed"), original.replace("Warning: first\n", ""),
                    original.replace("Warning: first\n", "Warning: first\nWarning: extra\n"),
                    original.replace("first\nWarning: second", "second\nWarning: first"),
                    original.replace("BEFORE", "AFTER"), original.replace("failure", "other")):
        assert diagnostic_layout(changed) != expected
    assert diagnostic_layout("ERROR: ‘sum’ not meaningful for ordered factors\n") == "ERROR: ‘sum’ not meaningful for ordered factors\n"


def fixture_cases() -> list[Case]:
    rows = FIXTURE.read_text().splitlines()
    assert "R version 4.6.0 " in rows[0] and "aarch64-apple-darwin" in rows[0] and rows[0].endswith("LDOUBLE 8"), rows[0]
    cases: list[Case] = []
    for row in rows[1:]:
        identifier, source, storage, kind, value, error, warnings = row.split("\t")
        if int(identifier) not in SELECTED:
            continue
        assert warnings == "-", row
        if error != "-":
            cases.append(Case(f"fixture-{identifier}", source, "", 1, "ERROR: " + bytes.fromhex(error).decode() + "\n"))
            continue
        checks = {"NA": "is.na(x)&!is.nan(x)", "NaN": "is.na(x)&is.nan(x)"}
        if kind not in checks:
            assert kind == "number", row
            number = struct.unpack(">d", bytes.fromhex(value))[0] if storage == "double" else int(value)
            literal = ("-Inf" if number < 0 else "Inf") if math.isinf(number) else float(number).hex()
            check = f"!is.na(x)&!is.nan(x)&x=={literal}"
            if number == 0:
                check += "&1/x==" + ("-Inf" if value == "8000000000000000" else "Inf")
        else:
            check = checks[kind]
        cases.append(Case(f"fixture-{identifier}", f'x<-{{{source}}};cat(typeof(x)=="{storage}"&({check}),"\\n",sep="")', "TRUE\n"))
    assert len(cases) == len(SELECTED), len(cases)
    return cases


def transport_cases() -> list[Case]:
    cases = [
        Case("dots-apply", 'f<-function(...)sum(...);print(f(1L,2L));g<-function(...)prod(...);print(g(2L,3L));print(apply(matrix(1:4,nrow=2),2,sum))', "[1] 3\n[1] 6\n[1] 3 7\n"),
        Case("eager-warning", 'options(warn=1);sum({cat("FIRST\\n");1},as.integer("bad"),{cat("LAST\\n");2},na.rm=TRUE);cat("AFTER\\n")', "FIRST\nLAST\n[1] 3\nAFTER\n", diagnostic="Warning: NAs introduced by coercion\n"),
        Case("warning-before-error", 'options(warn=1);prod(as.integer("bad"),na.rm=NA);cat("AFTER\\n")', "", 1, "Warning: NAs introduced by coercion\nERROR: NA in coercion to boolean\n"),
        Case("missing-stops-evaluation", 'sum(,{cat("LATE\\n");1});cat("AFTER\\n")', "", 1, "ERROR: argument 1 is empty\n"),
        Case("trailing-missing", 'prod(1,);cat("AFTER\\n")', "", 1, "ERROR: argument 2 is empty\n"),
        Case("duplicate-after-evaluation", 'sum({cat("DATA\\n");1},na.rm=TRUE,na.rm={cat("FLAG\\n");FALSE});cat("AFTER\\n")', "DATA\nFLAG\n", 1, 'ERROR: formal argument "na.rm" matched by multiple actual arguments\n'),
        Case("late-factor-storage", 'x<-sum(1L,factor("a"));y<-sum(NULL,factor("a"));z<-sum(1,na.rm=factor("a"));cat(x,typeof(x),y,typeof(y),z,typeof(z),"\\n")', "2 integer 1 integer 1 double \n"),
    ]
    for name, operand in (("sum", "-Inf"), ("prod", "0")):
        cases.append(Case(f"{name}-generated-nan", f'x<-{name}(c(Inf,{operand},NA_real_),na.rm=TRUE);cat(is.na(x),is.nan(x),typeof(x),"\\n")', "TRUE TRUE double \n"))
    return cases


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    comparator_self_check()
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT / "target/debug/r_ust_cli").resolve()
    environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL", LC_ALL="en_US.UTF-8", LANG="en_US.UTF-8")
    metadata = run(Invocation(("Rscript", "--vanilla", "-e", 'cat(R.version$major,R.version$minor,R.version$arch,.Machine$sizeof.longdouble,l10n_info()[["UTF-8"]],sep="/")'), b"", environment))
    assert metadata.status == 0 and metadata.stdout == "4/6.0/aarch64/8/TRUE" and not metadata.stderr, metadata
    paths = [*ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"), *ROOT.glob("crates/**/Cargo.toml"), ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__)]
    sources = {str(path.relative_to(ROOT)): digest(path) for path in paths}
    references = {str(Path(shutil.which(name) or name).resolve()): digest(Path(shutil.which(name) or name).resolve()) for name in ("R", "Rscript")}
    results: list[Comparison] = []
    cases = fixture_cases() + transport_cases()
    originals = (binary, binary.parent / "Rscript")
    original_hashes = {path.name: digest(path) for path in originals}
    with tempfile.TemporaryDirectory(prefix="r-ust-summaries-") as directory:
        snapshots = tuple(Path(directory) / name for name in ("r_ust_cli", "Rscript"))
        for original, snapshot in zip(originals, snapshots, strict=True):
            _ = shutil.copy2(original, snapshot)
        hashes = {path.name: digest(path) for path in snapshots}
        assert hashes == original_hashes, "binary changed during snapshot"
        for mode in MODES:
            for case in cases:
                source = Path(directory) / "summary source.R"
                _ = source.write_text(case.source + "\n")
                executions: list[Run] = []
                for executables in (("R", "Rscript"), (str(snapshots[0]), str(snapshots[1]))):
                    command, data = commands(mode, executables, source)
                    invocation = Invocation(command, data, environment)
                    executions.extend((run(invocation), run(invocation, merged=True)))
                reference, reference_merged, native, native_merged = executions
                passed = all(value.status == case.status for value in executions)
                passed &= reference.stdout == native.stdout == case.stdout
                passed &= diagnostic_layout(reference.stderr) == diagnostic_layout(native.stderr) == diagnostic_layout(case.diagnostic)
                passed &= diagnostic_layout(reference_merged.stdout) == diagnostic_layout(native_merged.stdout)
                results.append(Comparison(mode, case, reference, native, reference_merged, native_merged, passed,
                                          reference.raw_base64 == native.raw_base64, reference_merged.raw_base64 == native_merged.raw_base64))
            print(f"Checked {mode}", flush=True)
        binaries_stable = all(digest(path) == hashes[path.name] for path in (*snapshots, *originals))
    stable = all(digest(ROOT / path) == value for path, value in sources.items())
    reference_stable = all(digest(Path(path)) == value for path, value in references.items())
    report = ROOT / "target/summary-parity.json"
    _ = report.write_text(json.dumps({"reference_profile": asdict(metadata), "reference_sha256": references,
        "source_sha256": sources, "binary_sha256": hashes, "source_stable": stable,
        "binaries_stable": binaries_stable, "reference_stable": reference_stable,
        "comparator_self_check": "passed: changed, dropped, added, reordered diagnostics and stdout",
        "fixture_selection": SELECTED, "cases": [asdict(value) for value in results],
        "exact_success_comparisons": sum(row.passed and not row.case.diagnostic and row.exact_streams and row.exact_merged for row in results),
        "diagnostic_comparisons": sum(row.passed and bool(row.case.diagnostic) for row in results),
        "raw_stream_differences": sum(not row.exact_streams for row in results),
        "raw_merged_differences": sum(not row.exact_merged for row in results),
        "exclusions": ["Error prefix, call context, Calls trace and Execution halted layout only; raw streams retained.",
                       "Unordered-factor sum/prod diagnostic quote typography only; ordered-factor and message text unchanged.",
                       "NaN payload bits belong to evaluator fixtures; CLI checks missing classification and exact finite witnesses.",
                       "Only R 4.6.0 aarch64 binary64 LDOUBLE 8 UTF-8 profile; general S3/data-frame Summary, complex, raw and ALTREP remain Partial."]}, indent=2) + "\n")
    failures = [row for row in results if not row.passed]
    for row in failures:
        print(f"FAIL {row.mode}/{row.case.label}")
    print(f"{len(results) - len(failures)}/{len(results)} comparisons passed; manifests stable: {stable and binaries_stable and reference_stable}")
    return int(bool(failures) or not (stable and binaries_stable and reference_stable))


if __name__ == "__main__":
    raise SystemExit(main())
