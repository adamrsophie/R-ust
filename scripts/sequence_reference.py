#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/sequence_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex, program
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.sequence_capture import LIMIT_SOURCE
from scripts.sequence_capture import self_check as capture_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/sequence-r-4.6.0.tsv"
FIXTURE_SHA: Final = "3a257299869a9e7185acd5a22930e1e25f45d6f361ebb6451b6aac4080d5bbba"
LOGICAL_FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/logical-lists-r-4.6.0.tsv"
LOGICAL_SHA: Final = "45cdaf61d70c687cb89ec15f1e21a7466ca447dba5a8ffa5d0f07922505cf715"
HEADER: Final = "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 JIT 0 FRESH_PROCESS_PER_ROW"
LOGICAL_HEADER: Final = "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 JIT 0"


def parse_row(expected_id: int, line: str, *, logical: bool = False) -> Case:
    assert not any(character in line for character in "\r\n")
    if logical:
        identifier, source, storage, payload, error_hex, warnings_hex = line.split("\t")
        stdout_hex = "-"
    else:
        identifier, source, storage, payload, error_hex, warnings_hex, stdout_hex = line.split("\t")
    assert identifier == str(expected_id) and source
    assert storage in ("error", "logical" if logical else "integer")
    error = decode_hex(error_hex)
    warnings = tuple(decode_hex(warnings_hex).split("\n")) if warnings_hex != "-" else ()
    stdout = decode_hex(stdout_hex)
    assert all(warnings)
    if warnings and stdout:
        assert not logical and expected_id == 69 and source == LIMIT_SOURCE
    if storage == "error":
        assert error and payload == "-"
        fingerprint = "-"
    elif logical:
        assert not error
        values = () if payload == "-" else tuple(payload.split(","))
        assert all(value in ("T", "F", "NA") for value in values)
        fingerprint = f"logical|{len(values)}|{','.join(values)}"
        _ = literal(fingerprint)
    else:
        assert not error
        kind, count_text, data = payload.split("|", 2)
        assert kind == "integer" and re.fullmatch(r"0|[1-9][0-9]*", count_text)
        values = () if not data else tuple(data.split(","))
        assert len(values) == int(count_text)
        assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
        assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
        fingerprint = payload
        _ = literal(fingerprint)
    return Case(
        90 + expected_id if logical else expected_id,
        Lane.REPRESENTED,
        source,
        storage,
        fingerprint,
        error,
        warnings,
        stdout,
    )


def parse_fixtures(text: str, logical_text: str) -> tuple[Case, ...]:
    lines, logical_lines = text.splitlines(), logical_text.splitlines()
    assert len(lines) == 91 and lines[0] == HEADER
    assert len(logical_lines) == 25 and logical_lines[0] == LOGICAL_HEADER
    sequence = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    logical = tuple(parse_row(identifier, line, logical=True) for identifier, line in enumerate(logical_lines[1:], 1))
    assert Counter(row.storage for row in sequence) == {"integer": 47, "error": 43}
    assert Counter(row.storage for row in logical) == {"logical": 18, "error": 6}
    assert [row.identifier for row in sequence if row.warnings] == [
        18,
        19,
        20,
        21,
        24,
        44,
        45,
        46,
        67,
        68,
        69,
        79,
        89,
        90,
    ]
    assert [row.identifier for row in sequence if row.stdout] == [62, 63, 69, 87]
    assert [row.identifier for row in sequence if row.stdout and row.warnings] == [69]
    assert (sequence[61].stdout, sequence[62].stdout, sequence[86].stdout) == ("ANFBR", "NFBR", "N")
    assert sequence[68].source == LIMIT_SOURCE
    assert all(not row.warnings and not row.stdout and row.source.startswith("as.logical(") for row in logical)
    rows = sequence + logical
    assert tuple(row.identifier for row in rows) == tuple(range(1, 115))
    return rows


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA and digest(LOGICAL_FIXTURE) == LOGICAL_SHA
    return parse_fixtures(FIXTURE.read_text(), LOGICAL_FIXTURE.read_text())


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    capture_self_check(rows[68], rows[0])
    for row in rows:
        source = program(row)
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
    value = parse_row(1, "1\tx\tinteger\tinteger|3|1,NA,-7\t-\t-\t4142")
    assert value.fingerprint == "integer|3|1,NA,-7" and value.stdout == "AB"
    assert parse_row(1, "1\tx\tinteger\tinteger|0|\t-\t-\t-").fingerprint == "integer|0|"
    assert parse_row(1, "1\tx\tlogical\tT,F,NA\t-\t-", logical=True).fingerprint == "logical|3|T,F,NA"
    assert parse_row(1, "1\tx\tlogical\t-\t-\t-", logical=True).fingerprint == "logical|0|"
    for invalid in (
        "1\tx\tinteger\tinteger|1|1\t-\t-",
        "1\tx\tinteger\tinteger|1|1\t-\t-\t-\textra",
        "2\tx\tinteger\tinteger|1|1\t-\t-\t-",
        "01\tx\tinteger\tinteger|1|1\t-\t-\t-",
        "1\t\tinteger\tinteger|1|1\t-\t-\t-",
        "1\tx\tlogical\tlogical|1|T\t-\t-\t-",
        "1\tx\tinteger\tlogical|1|T\t-\t-\t-",
        "1\tx\tinteger\tinteger|01|1\t-\t-\t-",
        "1\tx\tinteger\tinteger|2|1\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|01\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|+1\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|-0\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|2147483648\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|-2147483648\t-\t-\t-",
        "1\tx\terror\tinteger|1|1\t626f6f6d\t-\t-",
        "1\tx\terror\t-\t-\t-\t-",
        "1\tx\terror\t-\t\t-\t-",
        "1\tx\terror\t-\tff\t-\t-",
        "1\tx\terror\t-\t626f6f6d\t66697273740a\t-",
        "1\tx\terror\t-\t626f6f6d\t\t-",
        "1\tx\terror\t-\t626f6f6d\t-\t5A",
        "1\tx\terror\t-\t626f6f6d\t-\tff",
        "1\tx\terror\t-\t626f6f6d\t61\t62",
        "1\tx\terror\t-\t626f6f6d\t-\t-\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    for invalid in (
        "1\tx\tlogical\tT\t-",
        "1\tx\tlogical\tT\t-\t-\textra",
        "2\tx\tlogical\tT\t-\t-",
        "1\tx\tinteger\t1\t-\t-",
        "1\tx\tlogical\tTRUE\t-\t-",
        "1\tx\tlogical\tT,\t-\t-",
        "1\tx\tlogical\t\t-\t-",
        "1\tx\tlogical\tT\t626f6f6d\t-",
        "1\tx\terror\tT\t626f6f6d\t-",
    ):
        try:
            _ = parse_row(1, invalid, logical=True)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    fixture, logical_fixture = FIXTURE.read_text(), LOGICAL_FIXTURE.read_text()
    for invalid, invalid_logical in (
        ("", logical_fixture),
        (fixture, ""),
        (fixture.replace(HEADER, "invalid", 1), logical_fixture),
        (fixture, logical_fixture.replace(LOGICAL_HEADER, "invalid", 1)),
        ("\n".join(fixture.splitlines()[:-1]), logical_fixture),
        (fixture, "\n".join(logical_fixture.splitlines()[:-1])),
        (fixture.replace("\n2\t", "\n1\t", 1), logical_fixture),
        (fixture, logical_fixture.replace("\n2\t", "\n1\t", 1)),
        (fixture.replace(LIMIT_SOURCE, "sequence(1)", 1), logical_fixture),
    ):
        try:
            _ = parse_fixtures(invalid, invalid_logical)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixtures accepted")


if __name__ == "__main__":
    self_check()
    print(
        "Sequence/logical readers and ordered/limit comparators passed; 114 attempted sources, 113 supported, 1 LIMIT"
    )
