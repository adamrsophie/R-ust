#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/run_length_reference.py
from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import self_check as literal_self_check
from scripts.run_length_values import Kind, parse_value, program
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/run-length-r-4.6.0.tsv"
FIXTURE_SHA: Final = "24976d978ad6044c30b4faf7b98f062554cb37b9b3ce9837371a4dcc21005c07"
BOUNDARY: Final = ROOT / "crates/r_ust_eval/tests/fixtures/run-length-boundary-r-4.6.0.tsv"
BOUNDARY_SHA: Final = "299dd253288f32bbf1b6c682e8948833bb041a25c02714e7fd4b083a6ba8cfe3"
HEADER: Final = "# R version 4.6.0 (2026-04-24); class rle omitted from canonical representation"
BOUNDARY_HEADER: Final = "id\tsource\tstatus\tvalue_or_error\tstdout_hex\twarnings_hex"
BOUNDARY_IDS: Final = (
    "unused_positional",
    "unused_dots",
    "unused_named",
    "alias_inverse",
    "alias_rle",
    "isolated_rle_helpers",
    "isolated_inverse_helper",
    "caller_x",
    "missing_caller_x",
    "closure_rle",
    "closure_inverse",
    "private_rle",
    "private_inverse",
    "force_x_not_dots",
    "force_once",
    "ordered_warnings",
)


def make_case(identifier: int, source: str, status: str, payload: str, warnings: tuple[str, ...], stdout: str) -> Case:
    assert source and status in ("OK", "ERR") and all(warnings)
    assert not (warnings and stdout), "ordered effects plus warnings need explicit events"
    if status == "ERR":
        assert payload
        return Case(identifier, Lane.REPRESENTED, source, "error", "-", payload, warnings, stdout)
    value = parse_value(payload)
    return Case(identifier, Lane.REPRESENTED, source, value.kind.value, payload, "", warnings, stdout)


def parse_row(identifier: int, line: str) -> Case:
    assert not any(character in line for character in "\r\n")
    source, status, payload, warnings_text = line.split("\t")
    warnings = tuple(warnings_text.split("|")) if warnings_text else ()
    return make_case(identifier, source, status, payload, warnings, "")


def parse_boundary(index: int, line: str) -> Case:
    assert not any(character in line for character in "\r\n")
    identifier, source, status, payload, stdout_hex, warnings_hex = line.split("\t")
    assert identifier == BOUNDARY_IDS[index]
    assert stdout_hex != "-" and warnings_hex != "-"
    stdout = decode_hex(stdout_hex) if stdout_hex else ""
    warnings = tuple(decode_hex(warnings_hex).split("\x1f")) if warnings_hex else ()
    return make_case(50 + index, source, status, payload, warnings, stdout)


def parse_fixtures(text: str, boundary_text: str) -> tuple[Case, ...]:
    lines, boundary_lines = text.splitlines(), boundary_text.splitlines()
    assert len(lines) == 50 and lines[0] == HEADER
    assert len(boundary_lines) == 17 and boundary_lines[0] == BOUNDARY_HEADER
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    boundaries = tuple(parse_boundary(index, line) for index, line in enumerate(boundary_lines[1:]))
    assert Counter(row.storage for row in rows) == {
        "list": 18,
        "integer": 6,
        "double": 2,
        "character": 2,
        "logical": 1,
        "factor": 1,
        "error": 19,
    }
    assert Counter(row.storage for row in boundaries) == {"integer": 8, "character": 2, "error": 6}
    result = rows + boundaries
    assert tuple(row.identifier for row in result) == tuple(range(1, 66))
    assert [(row.identifier, row.stdout) for row in result if row.stdout] == [(63, "[1] 7\n"), (64, "once")]
    assert [(row.identifier, len(row.warnings)) for row in result if row.warnings] == [(46, 1), (65, 2)]
    return result


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA and digest(BOUNDARY) == BOUNDARY_SHA
    return parse_fixtures(FIXTURE.read_text(), BOUNDARY.read_text())


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    for row in rows:
        source = program(row)
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
    negative_zero = parse_value("double:0000000000000080")
    assert negative_zero.fingerprint == "double|1|8000000000000000"
    assert parse_value("double:NA,NaN").fingerprint == "double|2|7ff00000000007a2,7ff8000000000000"
    assert parse_value('character:"a,b",NA,"🙂"').fingerprint == "character|3|x612c62,NA,xf09f9982"
    assert parse_value("list:=null").children[0].kind == Kind.NULL
    assert parse_value('factor:integer:1,1:character:"a"').children[0].fingerprint == "integer|2|1,1"
    for invalid in (
        "",
        "null:",
        "raw:00",
        "integer:01",
        "integer:+1",
        "integer:-0",
        "integer:1,",
        "integer:2147483648",
        "integer:-2147483648",
        "logical:true",
        "logical:1",
        "double:0",
        "double:000000000000F03F",
        "double:a20700000000f07f",
        "double:000000000000f87f",
        'character:"a",',
        'character:"a',
        'character:"\\u0000"',
        'character:"\\ud800"',
        "list:integer:1",
        "list:a",
        "list:1a=integer:1",
        "list:a=list:b=integer:1",
        "list:lengths=logical:TRUE;values=integer:1",
        "list:lengths=integer:1;values=unknown:1",
        'factor:logical:TRUE:character:"a"',
        'factor:integer:0:character:"a"',
        'factor:integer:2:character:"a"',
        "factor:integer:1:character:",
    ):
        try:
            _ = parse_value(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    for invalid in (
        "x\tOK\tinteger:1",
        "x\tOK\tinteger:1\t\textra",
        "\tOK\tinteger:1\t",
        "x\tMAYBE\tinteger:1\t",
        "x\tERR\t\t",
        "x\tOK\tinteger:1\tfirst|",
        "x\tOK\tinteger:1\t\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    for invalid in (
        "unused_positional\tx\tOK\tinteger:1\t",
        "unused_positional\tx\tOK\tinteger:1\t\t\textra",
        "wrong_id\tx\tOK\tinteger:1\t\t",
        "unused_positional\tx\tERR\t\t\t",
        "unused_positional\tx\tOK\tinteger:1\t-\t",
        "unused_positional\tx\tOK\tinteger:1\t\t-",
        "unused_positional\tx\tOK\tinteger:1\tff\t",
        "unused_positional\tx\tOK\tinteger:1\t\t61Ff",
        "unused_positional\tx\tERR\tboom\t\t611f",
        "unused_positional\tx\tERR\tboom\t61\t62",
    ):
        try:
            _ = parse_boundary(0, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    fixture, boundary = FIXTURE.read_text(), BOUNDARY.read_text()
    for invalid, invalid_boundary in (
        ("", boundary),
        (fixture, ""),
        (fixture.replace(HEADER, "# invalid", 1), boundary),
        (fixture, boundary.replace(BOUNDARY_HEADER, "invalid", 1)),
        ("\n".join(fixture.splitlines()[:-1]), boundary),
        (fixture, "\n".join(boundary.splitlines()[:-1])),
        (fixture, boundary.replace("unused_positional\t", "unused_dots\t", 1)),
        (fixture, boundary.replace("\t6f6e6365\t", "\t616761696e\t", 1)),
    ):
        try:
            _ = parse_fixtures(invalid, invalid_boundary)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixtures accepted")


if __name__ == "__main__":
    self_check()
    print("Run-length reader, program and shared comparator self-checks passed; 49 + 16 fixture rows validated")
