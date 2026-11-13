#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# Run: uv run scripts/check-closure-matching.py [target/debug/r_ust_cli]
# Or: python3 scripts/check-closure-matching.py [target/debug/r_ust_cli]
"""Compare closure transports against the independent R 4.6.0 JIT0 fixtures."""
from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Iterator
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, Literal, assert_never

Mode = Literal["expression", "file", "stdin", "rscript-file", "rscript-expression"]
MODES: Final[tuple[Mode, ...]] = ("expression", "file", "stdin", "rscript-file", "rscript-expression")
ROOT: Final = Path(__file__).resolve().parent.parent
FIXTURES: Final = ROOT / "crates/r_ust_eval/tests/fixtures"
SPECS: Final = (
    ("closure-matching-r-4.6.0.tsv", "d05a2c42385438d24cc1f1c423dba453265285ccff5ec99e2a75e7e4b93f76c1",
     (14, 117, 121, 282, 399, 1073, 1074, 1075, 1076, 1078, 1079, 1080, 1087, 1095, 1167,
      1185, 1186, 1188, 1189, 1190, 1191, 1193, 1195, 1197, 1198, 1199, 1200, 1201, 1203, 1204, 1207, 1209)),
    ("promise-control-r-4.6.0.tsv", "ddf1df99fc096f31cffbe661b4c1f813faf9def9e88ba5d65b21630304ca2a99",
     (8, 10, 11, 15, 24, 25, 27, 29, 31, 32, 33, 34, 35, 38, 43, 45)),
)


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
        case "file":
            return (executables[0], "--vanilla", "--no-echo", "-f", str(source)), b""
        case "rscript-file":
            return (executables[1], "--vanilla", str(source)), b""
        case "expression" | "rscript-expression":
            prefix = (executables[0], "--vanilla", "--no-echo") if mode == "expression" else (executables[1], "--vanilla")
            return (*prefix, "-e", source.read_text().replace("\\", "\\\\")), b""
        case _:
            assert_never(mode)


def diagnostic_layout(text: str) -> str:
    """Preserve condition text and order while removing enumerated CLI call layout."""
    text = re.sub(r"(?m)^(?:Execution halted|Calls: [^\n]*)\n", "", text)
    text = re.sub(r"(?m)^(?:Error in [^\n]*? :[ \t]*(?:\n  )?|Error: |(?:argument|domain|control-flow) error: )", "ERROR: ", text)
    text = re.sub(r"(?m)^(object '[^\n]+' not found|argument '[^\n]*' is missing, with no default)$", r"ERROR: \1", text)
    text = re.sub(r"(?m)^(?:In addition: )?Warning messages?:\n", "", text)
    text = re.sub(r"(?m)^(?:Warning in [^\n]*? :|(?:\d+: )?In [^\n]*? :)[ \t]*(?:\n  )?", "WARNING: ", text)
    warning = r"(partial argument match of '[^'\n]+' to '[^'\n]+'|restarting interrupted promise evaluation)"
    return re.sub(r"(?m)^(?:WARNING: |Warning: |\d+: )?" + warning + r" ?$", r"WARNING: \1", text)


def comparator_self_check() -> None:
    first, second = "partial argument match of 'a' to 'alpha'", "partial argument match of 'b' to 'beta'"
    raw = f"BEFORE\nWarning in f(a = 1) : {first}\nWarning in f(b = 2) : {second}\nError in f() : unused argument (x = 1)\nExecution halted\n"
    expected = f"BEFORE\nWARNING: {first}\nWARNING: {second}\nERROR: unused argument (x = 1)\n"
    assert diagnostic_layout(raw) == expected
    for changed in (raw.replace(first, second), raw.replace(f"Warning in f(a = 1) : {first}\n", ""),
                    raw.replace("BEFORE", "AFTER"), raw.replace("(x = 1)", "(x = 2)"),
                    raw.replace(first, "SWAP").replace(second, first).replace("SWAP", second),
                    raw.replace("BEFORE\n", f"BEFORE\nWarning: {first}\n")):
        assert diagnostic_layout(changed) != expected
    assert diagnostic_layout("ERROR: unused argument (x = {\n    1\n})\n") != "ERROR: unused argument (x = { 1 })\n"


def fixture_literal(tokens: Iterator[str]) -> str:
    match next(tokens):
        case "NULL":
            return "NULL"
        case "integer":
            count, values = int(next(tokens)), next(tokens)
            numbers = [str(int(value)) + "L" for value in values.split(",")] if values else []
            assert len(numbers) == count
            return "c(" + ",".join(numbers) + ")" if numbers else "as.integer(NULL)"
        case "list":
            count = int(next(tokens))
            if count == 0:
                assert next(tokens) == ""
                return "list()"
            elements: list[str] = []
            for _ in range(count):
                name = next(tokens)
                tag = "" if name == "_" else bytes.fromhex(name).decode()
                assert not tag or re.fullmatch(r"[A-Za-z][A-Za-z0-9._]*", tag), tag
                elements.append((tag + "=" if tag else "") + fixture_literal(tokens))
            return "list(" + ",".join(elements) + ")"
        case kind:
            raise AssertionError(f"unsupported selected fixture type: {kind}")


def fixture_cases(path: Path, selected: tuple[int, ...]) -> list[Case]:
    rows = path.read_text().splitlines()
    assert "R version 4.6.0 " in rows[0] and "aarch64-" in rows[0] and rows[0].endswith("LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"), rows[0]
    cases: list[Case] = []
    for row in rows[1:]:
        identifier, source, storage, fingerprint, error, warnings, output = row.split("\t")
        if int(identifier) not in selected:
            continue
        stdout = "" if output == "-" else bytes.fromhex(output).decode()
        warning_text = "" if warnings == "-" else "".join("WARNING: " + value + "\n" for value in bytes.fromhex(warnings).decode().split("\n"))
        error_text = "" if error == "-" else "ERROR: " + bytes.fromhex(error).decode() + "\n"
        if storage != "error":
            tokens = iter(re.split(r"[|;=]", fingerprint))
            expected = fixture_literal(tokens)
            assert next(tokens, None) is None, fingerprint
            source = f'answer<-{{{source}}};cat(identical(answer,{expected}),"\\n",sep="")'
            stdout += "TRUE\n"
        else:
            source += ';cat("AFTER\\n")'
        diagnostic = warning_text + error_text if "warn=1" in source else error_text + warning_text
        cases.append(Case(f"{path.stem}-{identifier}", source, stdout, int(storage == "error"), diagnostic))
    assert len(cases) == len(selected), path
    return cases


def transport_cases() -> list[Case]:
    cases = [
        Case("binding-prints-once", "f<-function(alpha,beta)c(alpha,beta);f(1,alpha=2)", "[1] 2 1\n"),
        Case("shared-promise-once", 'g<-function(x)x+x;f<-function(x)g(x);f({cat("ONCE\\n");3L})', "ONCE\n[1] 6\n"),
        Case("cached-unused-promise", 'g<-function()0;f<-function(...){x<-list(...);g(...)};f(unused={cat("ONCE\\n");9L});cat("AFTER\\n")', "ONCE\n", 1, "ERROR: unused argument (unused = 9)\n"),
        Case("direct-unused-print", 'g<-function()0;g(unused=print(9));cat("AFTER\\n")', "", 1, "ERROR: unused argument (unused = print(9))\n"),
    ]
    for enabled, level in ((False, 2), (True, -1), (True, 0), (True, 1), (True, 2)):
        source = f'options(warnPartialMatchArgs={str(enabled).upper()},warn={level});f<-function(alpha){{cat("BODY\\n");alpha}};cat("BEFORE\\n");f(al={{cat("ARG\\n");2L}});cat("AFTER\\n")'
        warning = "partial argument match of 'al' to 'alpha'\n"
        promoted = enabled and level == 2
        diagnostic = "ERROR: (converted from warning) " + warning if promoted else ("WARNING: " + warning if enabled and level >= 0 else "")
        cases.append(Case(f"warning-{enabled}-{level}", source, "BEFORE\n" + ("" if promoted else "BODY\nARG\n[1] 2\nAFTER\n"), int(promoted), diagnostic))
    return cases


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    comparator_self_check()
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT / "target/debug/r_ust_cli").resolve()
    environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL", R_ENABLE_JIT="0", LC_ALL="en_US.UTF-8", LANG="en_US.UTF-8")
    metadata = run(Invocation(("Rscript", "--vanilla", "-e", 'cat(R.version$major,R.version$minor,R.version$arch,.Machine$sizeof.longdouble,compiler::enableJIT(-1L),sep="/")'), b"", environment))
    assert metadata.status == 0 and metadata.stdout == "4/6.0/aarch64/8/0" and not metadata.stderr, metadata
    assert all(digest(FIXTURES / name) == expected for name, expected, _ in SPECS), "independent fixture hash mismatch"
    paths = [*ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"), *ROOT.glob("crates/**/Cargo.toml"), ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__)]
    sources = {str(path.relative_to(ROOT)): digest(path) for path in paths}
    references = {str(Path(shutil.which(name) or name).resolve()): digest(Path(shutil.which(name) or name).resolve()) for name in ("R", "Rscript")}
    cases = [case for name, _, selected in SPECS for case in fixture_cases(FIXTURES / name, selected)] + transport_cases()
    results: list[Comparison] = []
    originals = (binary, binary.parent / "Rscript")
    hashes = {path.name: digest(path) for path in originals}
    with tempfile.TemporaryDirectory(prefix="r-ust-closure-") as directory:
        snapshots = tuple(Path(directory) / name for name in ("r_ust_cli", "Rscript"))
        for original, snapshot in zip(originals, snapshots, strict=True):
            _ = shutil.copy2(original, snapshot)
        assert all(digest(path) == hashes[path.name] for path in snapshots), "binary changed during snapshot"
        for mode in MODES:
            for case in cases:
                source = Path(directory) / "closure source.R"
                _ = source.write_text(case.source + "\n")
                executions: list[Run] = []
                for executables in (("R", "Rscript"), (str(snapshots[0]), str(snapshots[1]))):
                    command, data = commands(mode, executables, source)
                    invocation = Invocation(command, data, environment)
                    executions.extend((run(invocation), run(invocation, merged=True)))
                reference, reference_merged, native, native_merged = executions
                passed = all(value.status == case.status for value in executions) and reference.stdout == native.stdout == case.stdout
                passed &= diagnostic_layout(reference.stderr) == diagnostic_layout(native.stderr) == case.diagnostic
                passed &= diagnostic_layout(reference_merged.stdout) == diagnostic_layout(native_merged.stdout)
                results.append(Comparison(mode, case, reference, native, reference_merged, native_merged, passed,
                                          reference.raw_base64 == native.raw_base64, reference_merged.raw_base64 == native_merged.raw_base64))
            print(f"Checked {mode}", flush=True)
        binaries_stable = all(digest(path) == hashes[path.name] for path in (*snapshots, *originals))
    stable = all(digest(ROOT / path) == value for path, value in sources.items()) and all(digest(Path(path)) == value for path, value in references.items())
    report = ROOT / "target/closure-matching-parity.json"
    _ = report.write_text(json.dumps({"reference_profile": asdict(metadata), "reference_sha256": references,
        "source_sha256": sources, "binary_sha256": hashes, "sources_and_references_stable": stable, "binaries_stable": binaries_stable,
        "comparator_self_check": "passed: changed, dropped, added, reordered conditions/events and changed multiline grammar",
        "fixture_selection": SPECS, "cases": [asdict(value) for value in results],
        "exact_success_comparisons": sum(row.passed and not row.case.diagnostic and row.exact_streams and row.exact_merged for row in results),
        "diagnostic_comparisons": sum(row.passed and bool(row.case.diagnostic) for row in results),
        "raw_stream_differences": sum(not row.exact_streams for row in results), "raw_merged_differences": sum(not row.exact_merged for row in results),
        "exclusions": ["Condition call headers, native error prefixes, Calls and Execution halted; complete raw streams retained.",
                       "Warning headings/call prefixes/number labels and one deferred pad space on the two exact known warning forms.",
                       "Nested list display is not compared; fixture-derived identical() checks preserve type, values and names.",
                       "R 4.6.0 aarch64 LDOUBLE8 JIT0 only. JIT3 control cases 24-26 remain separate compiler-profile work."]}, indent=2) + "\n")
    failures = [row for row in results if not row.passed]
    for row in failures:
        print(f"FAIL {row.mode}/{row.case.label}")
    print(f"{len(results) - len(failures)}/{len(results)} comparisons passed; manifests stable: {stable and binaries_stable}")
    return int(bool(failures) or not (stable and binaries_stable))


if __name__ == "__main__":
    raise SystemExit(main())
