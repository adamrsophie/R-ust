#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/bitwise_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import dataclass, replace
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex
from scripts.constructor_reference import matches as shared_matches
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import (
    Invocation,
    Run,
    comparator_self_check,
    digest,
    encode_bytes,
)

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/bitwise-r-4.6.0.tsv"
FIXTURE_SHA: Final = "92522e4934f960f6d1cafc7ea29f8f46fee36c38c68ebd4525fb9f38e2e76a8c"
HEADER: Final = "id\tname\tsource\ttype\tvalues\tvisible\terror_hex\twarnings_hex\tstdout_hex"
EFFECT_BEFORE_WARNING: Final = (
    (35, "bitwAnd(a=Inf,b={cat('B');1L})"),
    (86, "bitwOr(a=Inf,b={cat('B');1L})"),
    (137, "bitwXor(a=Inf,b={cat('B');1L})"),
    (188, "bitwShiftL(a=Inf,n={cat('B');1L})"),
    (239, "bitwShiftR(a=Inf,n={cat('B');1L})"),
)


class Storage(StrEnum):
    INTEGER = "integer"
    ERROR = "error"


@dataclass(frozen=True, slots=True)
class BitwiseCase(Case):
    name: str
    visible: str


def parse_row(expected_id: int, line: str) -> BitwiseCase:
    assert 1 <= expected_id <= 302 and not any(character in line for character in "\r\n")
    identifier, name, source, storage_text, payload, visible, error_hex, warnings_hex, stdout_hex = line.split("\t")
    assert identifier == str(expected_id) and source and ".bitwise_answer" not in source
    assert re.fullmatch(r"[A-Za-z][A-Za-z0-9_]*", name)
    error, stdout = decode_hex(error_hex), decode_hex(stdout_hex)
    warnings = tuple(decode_hex(warnings_hex).split("\n")) if warnings_hex != "-" else ()
    assert all(warnings)
    storage = Storage(storage_text)
    match storage:
        case Storage.ERROR:
            assert error and payload == "-" and visible == "-"
            fingerprint = "-"
        case Storage.INTEGER:
            assert not error and visible in ("TRUE", "FALSE")
            values = () if payload == "-" else tuple(payload.split(","))
            assert all(re.fullmatch(r"NA|0|-?[1-9][0-9]*", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
            fingerprint = f"integer|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
        case _:
            assert_never(storage)
    return BitwiseCase(
        expected_id, Lane.REPRESENTED, source, storage.value, fingerprint, error, warnings, stdout, name, visible
    )


def parse_fixture(text: str) -> tuple[BitwiseCase, ...]:
    assert "\r" not in text and text.endswith("\n")
    lines = text.splitlines()
    assert len(lines) == 303 and lines[0] == HEADER
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert len({row.name for row in rows}) == 302
    assert Counter(row.storage for row in rows) == {"integer": 162, "error": 140}
    assert Counter(row.visible for row in rows) == {"TRUE": 151, "FALSE": 11, "-": 140}
    assert sum(bool(row.warnings) for row in rows) == 34 and sum(bool(row.stdout) for row in rows) == 62
    assert [(row.identifier, row.source, row.stdout) for row in rows if row.warnings and row.stdout] == [
        (identifier, source, "B") for identifier, source in EFFECT_BEFORE_WARNING
    ]
    return rows


def read_cases() -> tuple[BitwiseCase, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    return parse_fixture(FIXTURE.read_text())


R_TERMINATED: Final = tuple((row.identifier, row.source, row.error) for row in read_cases() if row.error.endswith("\n"))
assert len(R_TERMINATED) == 21


def program(row: BitwiseCase) -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    return (
        "options(warn=1);.bitwise_answer<-withVisible({"
        + row.source
        + "});cat(identical(.bitwise_answer$value,"
        + literal(row.fingerprint)
        + ") && identical(.bitwise_answer$visible,"
        + row.visible
        + '),"\\n",sep="")\n'
    )


def matches(row: BitwiseCase, runs: tuple[Run, Run], *, reference: bool) -> bool:
    if reference and any(row.identifier == identifier for identifier, _, _ in R_TERMINATED):
        if (row.identifier, row.source, row.error) not in R_TERMINATED:
            return False
        row = replace(row, error=row.error.removesuffix("\n"))
    if row.warnings and row.stdout:
        if (row.identifier, row.source) not in EFFECT_BEFORE_WARNING or row.stdout != "B":
            return False
        separate, merged = runs
        if not separate.stdout.startswith(row.stdout) or not merged.stdout.startswith(row.stdout):
            return False
        runs = (
            replace(separate, stdout=separate.stdout[len(row.stdout) :]),
            replace(merged, stdout=merged.stdout[len(row.stdout) :]),
        )
        row = replace(row, stdout="")
    return shared_matches(row, runs, reference=reference)


def sample_run(stdout: str, stderr: str, status: int) -> Run:
    return Run(
        Invocation(("fixture-self-check",), b""),
        status,
        stdout,
        stderr,
        encode_bytes(stdout.encode()),
        encode_bytes(stderr.encode()),
    )


def effect_self_check(rows: tuple[BitwiseCase, ...]) -> None:
    for row in rows:
        for reference in (False, True):
            warnings = "".join("Warning: " + warning + "\n" for warning in row.warnings)
            error = "Error: " + row.error + "\n" if row.error else ""
            if reference and (row.identifier, row.source, row.error) in R_TERMINATED:
                error = error.removesuffix("\n")
            footer = "Execution halted\n" if row.error and reference else ""
            value = "" if row.error else "TRUE\n"
            separate = sample_run(row.stdout + value, warnings + error + footer, int(bool(row.error)))
            merged = sample_run(row.stdout + warnings + value + error + footer, "", separate.status)
            assert matches(row, (separate, merged), reference=reference), row.identifier
            if reference and row.error.endswith("\n"):
                assert not matches(replace(row, error=row.error[:-1]), (separate, merged), reference=True)
            if row.warnings and row.stdout:
                assert not matches(replace(row, identifier=row.identifier + 1), (separate, merged), reference=reference)
                assert not matches(replace(row, source=row.source + " "), (separate, merged), reference=reference)
            changes = [
                (replace(separate, status=1 - separate.status), merged),
                (separate, replace(merged, status=1 - merged.status)),
                (replace(separate, stdout=separate.stdout + "AFTER\n"), merged),
                (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
                (separate, replace(merged, stderr="unexpected\n")),
            ]
            if row.stdout:
                changes.extend(
                    (
                        (replace(separate, stdout="X" + separate.stdout[1:]), merged),
                        (separate, replace(merged, stdout="X" + merged.stdout[1:])),
                        (separate, replace(merged, stdout=merged.stdout[len(row.stdout) :])),
                    )
                )
            if row.warnings:
                changes.append((replace(separate, stderr=error + footer), merged))
                changes.append((separate, replace(merged, stdout=row.stdout + value + error + footer + warnings)))
                if row.stdout:
                    changes.append((separate, replace(merged, stdout=warnings + row.stdout + value + error + footer)))
            if row.error:
                changes.append((replace(separate, stderr=warnings + "Error: changed\n" + footer), merged))
            else:
                changes.append((replace(separate, stdout=row.stdout + "FALSE\n"), merged))
            for changed in changes:
                assert not matches(row, changed, reference=reference), (row.identifier, changed)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    effect_self_check(rows)
    for row in rows:
        source = program(row)
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
        if not row.error:
            assert f"identical(.bitwise_answer$value,{literal(row.fingerprint)})" in source
            assert f"&& identical(.bitwise_answer$visible,{row.visible})" in source
            assert "withVisible({" + row.source + "})" in source
    fixture = FIXTURE.read_text()
    lines = fixture.splitlines()
    for payload in ("-", "-2147483647,NA,2147483647"):
        row = parse_row(1, f"1\tboundaries\tbitwNot(0L)\tinteger\t{payload}\tFALSE\t-\t-\t-")
        assert row.visible == "FALSE" and row.fingerprint.startswith("integer|")
    for column, replacement in (
        (0, "0"),
        (0, "2"),
        (0, "01"),
        (1, ""),
        (1, "bad-name"),
        (2, ""),
        (3, "logical"),
        (4, ""),
        (4, "NaN"),
        (4, "01"),
        (4, "+1"),
        (4, "-0"),
        (4, "1,"),
        (4, "2147483648"),
        (4, "-2147483648"),
        (5, "-"),
        (5, "true"),
        (6, "626f6f6d"),
        (7, "66697273740a"),
        (7, "0a6669727374"),
        (7, ""),
        (8, ""),
        (8, "4A"),
        (8, "0"),
        (8, "ff"),
    ):
        fields = lines[1].split("\t")
        fields[column] = replacement
        try:
            _ = parse_row(1, "\t".join(fields))
        except (AssertionError, ValueError):
            continue
        raise AssertionError((column, replacement))
    error_row = next(row for row in rows if row.error)
    for column, replacement in ((4, "1"), (5, "TRUE"), (6, "-"), (6, ""), (6, "ff"), (6, "AA")):
        fields = lines[error_row.identifier].split("\t")
        fields[column] = replacement
        try:
            _ = parse_row(error_row.identifier, "\t".join(fields))
        except (AssertionError, ValueError):
            continue
        raise AssertionError((column, replacement))
    for invalid in (
        "",
        fixture.rstrip("\n"),
        fixture.replace(HEADER, "invalid", 1),
        fixture + lines[1] + "\n",
        "\n".join(lines[:-1]) + "\n",
        fixture.replace("\n", "\r\n"),
        fixture.replace("\n2\t", "\n1\t", 1),
        fixture.replace("\tTRUE\t", "\tFALSE\t", 1),
        fixture.replace("\t42\n", "\t-\n", 1),
    ):
        try:
            _ = parse_fixture(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")
    for invalid in (lines[1] + "\textra", lines[1] + "\n", "\t".join(lines[1].split("\t")[:-1])):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid row accepted")


if __name__ == "__main__":
    self_check()
    print("Bitwise reader, program and ordered comparator self-checks passed; 302 supported source rows")
