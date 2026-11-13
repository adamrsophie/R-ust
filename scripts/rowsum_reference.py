#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/rowsum_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.rowsum_values import Frame, parse_value, program, size
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/rowsum-r-4.6.0.tsv"
FIXTURE_SHA: Final = "8c90bd26a0efde2b83491231234f66f3fa1d48539eb78b7bf22895532939c9e6"
HEADER: Final = "id\tsource\tvalue\terror\twarning"
NAMES_FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/rowsum-column-names-r-4.6.0.tsv"
NAMES_SHA: Final = "a792cb5400580ba69fc0ebb6e146abbcbf954942e5f24edecf33a613b5821da9"
NAMES_HEADER: Final = "# R version 4.6.0 (2026-04-24) LC_COLLATE=C rowsum column names"
NAMES_COLUMNS: Final = "id\tsource\tnames"


def parse_row(identifier: int, line: str) -> Case:
    assert not any(character in line for character in "\r\n")
    expected_id, source, payload, error, warnings_text = line.split("\t")
    assert expected_id == str(identifier) and source and all((payload, error, warnings_text))
    warnings = tuple(warnings_text.split("|")) if warnings_text != "-" else ()
    assert all(warnings)
    if error != "-":
        assert payload == "-"
        return Case(identifier, Lane.REPRESENTED, source, "error", "-", error, warnings, "")
    value = parse_value(payload)
    storage = "df" if isinstance(value, Frame) else value.data.storage.value
    return Case(identifier, Lane.REPRESENTED, source, storage, payload, "", warnings, "")


def parse_fixture(text: str) -> tuple[Case, ...]:
    lines = text.splitlines()
    assert len(lines) == 45 and lines[0] == HEADER
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {
        "integer": 21,
        "double": 4,
        "df": 2,
        "logical": 1,
        "character": 2,
        "list": 1,
        "error": 13,
    }
    assert [row.identifier for row in rows if row.warnings] == [5, 6, 19]
    assert all(row.source.startswith("matrix(") for row in rows[37:])
    return rows


def parse_names(text: str, rows: tuple[Case, ...]) -> dict[int, str]:
    lines = text.splitlines()
    assert len(lines) == 4 and lines[:2] == [NAMES_HEADER, NAMES_COLUMNS]
    result: dict[int, str] = {}
    for expected_id, line in zip((13, 14), lines[2:], strict=True):
        identifier, source, fingerprint = line.split("\t")
        assert identifier == str(expected_id) and source == rows[expected_id - 1].source
        kind, count_text, payload = fingerprint.split("|", 2)
        expected = parse_value(rows[expected_id - 1].fingerprint)
        assert isinstance(expected, Frame) and kind == "character"
        count = size(count_text)
        values = () if not payload else tuple(payload.split(","))
        assert count == len(values) == len(expected.columns)
        for value in values:
            assert re.fullmatch(r"x(?:[0-9a-f]{2})*", value)
            assert "\0" not in bytes.fromhex(value[1:]).decode()
        _ = literal(fingerprint)
        result[expected_id] = fingerprint
    return result


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    return parse_fixture(FIXTURE.read_text())


def read_names(rows: tuple[Case, ...]) -> dict[int, str]:
    assert digest(NAMES_FIXTURE) == NAMES_SHA
    return parse_names(NAMES_FIXTURE.read_text(), rows)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    names = read_names(rows)
    assert set(names) == {13, 14}
    for row in rows:
        source = program(row, names.get(row.identifier, ""))
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
    assert parse_row(1, "1\tx\tinteger:1,1:7\t-\t-").storage == "integer"
    error = parse_row(1, "1\tx\t-\tboom\tfirst|second")
    assert error.error == "boom" and error.warnings == ("first", "second")
    for invalid in (
        "",
        "integer:2:1,2",
        "integer:2,2:1,2",
        "integer:0,2:1",
        "integer:01,1:1",
        "integer:-1,2:",
        "integer:2147483648,0:",
        "integer:1,1:01",
        "integer:1,1:+1",
        "integer:1,1:-0",
        "integer:1,1:2147483648",
        "integer:1,1:-2147483648",
        "logical:1,1:true",
        "logical:1,1:1",
        "double:1,1:NaNfoo",
        "double:1,1:1e999",
        "double:1,1:01",
        "double:1,1:0,",
        "character:1,1:a",
        "list:1,1:NULL",
        "raw:0,2:",
        "df:2:integer:1",
        "df:2:logical:TRUE,FALSE",
        "df:-1:integer:",
    ):
        try:
            _ = parse_value(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    for invalid in (
        "1\tx\tinteger:1,1:1\t-",
        "1\tx\tinteger:1,1:1\t-\t-\textra",
        "2\tx\tinteger:1,1:1\t-\t-",
        "01\tx\tinteger:1,1:1\t-\t-",
        "1\t\tinteger:1,1:1\t-\t-",
        "1\tx\tinteger:1,1:1\tboom\t-",
        "1\tx\t-\t-\t-",
        "1\tx\t-\t\t-",
        "1\tx\t-\tboom\t",
        "1\tx\t-\tboom\tfirst|",
        "1\tx\t-\tboom\t-\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    fixture = FIXTURE.read_text()
    for invalid in (
        "",
        fixture.replace(HEADER, "invalid", 1),
        "\n".join(fixture.splitlines()[:-1]),
        fixture + fixture.splitlines()[-1] + "\n",
        fixture.replace("\n2\t", "\n1\t", 1),
    ):
        try:
            _ = parse_fixture(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")
    supplement = NAMES_FIXTURE.read_text()
    for invalid in (
        "",
        supplement.replace(NAMES_HEADER, "invalid", 1),
        supplement.replace(NAMES_COLUMNS, "invalid", 1),
        "\n".join(supplement.splitlines()[:-1]),
        supplement.replace("13\t", "14\t", 1),
        supplement.replace(rows[12].source, "rowsum(1,1)", 1),
        supplement.replace("character|2|", "character|3|", 1),
        supplement.replace("character|2|", "logical|2|", 1),
        supplement.replace("x61,x62", "x61", 1),
        supplement.replace("x61,x62", "x61,NA", 1),
        supplement.replace("x61,x62", "xff,x62", 1),
        supplement.replace("x61,x62", "x6A,x62", 1),
        supplement.replace("x61,x62", "x00,x62", 1),
    ):
        try:
            _ = parse_names(invalid, rows)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid names supplement accepted")


if __name__ == "__main__":
    self_check()
    print("Rowsum reader, program and shared comparator self-checks passed; 44 cases and 2 overlapping names witnesses")
