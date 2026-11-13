#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/partial_match_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/partial-match-r-4.6.0.tsv"
FIXTURE_SHA: Final = "2269d824d6e90050f09ddbb9e9cf2b11821f42224ebdddb8a4459f75e4974814"
HEADER: Final = "# R version 4.6.0 (2026-04-24)"


class Storage(StrEnum):
    INTEGER = "integer"
    ERROR = "error"


def parse_row(expected_id: int, line: str) -> Case:
    identifier, source, storage_text, payload, error, warnings_text = line.split("\t")
    assert identifier == str(expected_id) and source
    warnings = () if warnings_text == "-" else tuple(warnings_text.split("|"))
    assert all(warnings) and not any(character in line for character in "\r\n")
    storage = Storage(storage_text)
    match storage:
        case Storage.ERROR:
            assert bool(error) and error != "-" and payload == "-"
            fingerprint = "-"
        case Storage.INTEGER:
            assert error == "-"
            values = () if payload == "-" else tuple(payload.split(","))
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
            fingerprint = f"integer|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
            error = ""
        case _:
            assert_never(storage)
    return Case(expected_id, Lane.REPRESENTED, source, storage.value, fingerprint, error, warnings, "")


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == HEADER and len(lines) == 66
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {"integer": 48, "error": 17}
    assert sum(bool(row.warnings) for row in rows) == 4
    return rows


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    _ = read_cases()
    value = parse_row(1, '1\tpmatch("a","ab")\tinteger\t1,NA,-7\t-\t-')
    assert value.fingerprint == "integer|3|1,NA,-7"
    assert literal(value.fingerprint) == "c(1L,NA_integer_,-7L)"
    assert parse_row(1, "1\tcharmatch(NULL,NULL)\tinteger\t-\t-\t-").fingerprint == "integer|0|"
    error = parse_row(1, "1\tpmatch(1,1)\terror\t-\tboom\tfirst|second")
    assert error.error == "boom" and error.warnings == ("first", "second")
    for invalid in (
        "1\tx\tinteger\t1\t-\t-\textra",
        "2\tx\tinteger\t1\t-\t-",
        "1\t\tinteger\t1\t-\t-",
        "1\tx\tlogical\tTRUE\t-\t-",
        "1\tx\tinteger\t1\tboom\t-",
        "1\tx\tinteger\t\t-\t-",
        "1\tx\tinteger\tNaN\t-\t-",
        "1\tx\tinteger\t2147483648\t-\t-",
        "1\tx\tinteger\t-2147483648\t-\t-",
        "1\tx\terror\t1\tboom\t-",
        "1\tx\terror\t-\t-\t-",
        "1\tx\terror\t-\t\t-",
        "1\tx\terror\t-\tboom\tfirst|",
        "1\tx\terror\t-\tboom\t\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)


if __name__ == "__main__":
    self_check()
    print("Partial-match reader and shared comparator self-checks passed; 65 fixture rows validated")
