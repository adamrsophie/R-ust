#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/unicode_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/unicode-conversion-r-4.6.0.tsv"
FIXTURE_SHA: Final = "c78d6e9dab7ea05915ee173d587edb8340ccb375725767a85464d9998d09583c"
HEADER: Final = "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 UNICODE_CONVERSION_V1"


class Storage(StrEnum):
    INTEGER = "integer"
    CHARACTER = "character"
    LOGICAL = "logical"
    ERROR = "error"


def parse_row(expected_id: int, line: str) -> Case:
    identifier, source, storage_text, fingerprint, error_hex, warnings_hex, stdout_hex = line.split("\t")
    assert identifier == str(expected_id) and source
    assert not any(character in line for character in "\r\n")
    error, warning_text, stdout = (decode_hex(field) for field in (error_hex, warnings_hex, stdout_hex))
    warnings: tuple[str, ...] = ()
    if warning_text:
        assert warning_text.startswith("Warning: ") and warning_text.endswith("\n")
        warnings = tuple(warning_text[len("Warning: ") : -1].split("\nWarning: "))
        assert all(warnings)
    storage = Storage(storage_text)
    match storage:
        case Storage.ERROR:
            assert error and fingerprint == "-"
        case Storage.INTEGER | Storage.CHARACTER | Storage.LOGICAL:
            assert not error
            kind, count, payload = fingerprint.split("|")
            assert kind == storage.value and re.fullmatch(r"(?:0|[1-9][0-9]*)", count)
            values = tuple(payload.split(",")) if payload else ()
            assert len(values) == int(count)
            match storage:
                case Storage.INTEGER:
                    assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
                    assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
                case Storage.CHARACTER:
                    assert all(value == "NA" or re.fullmatch(r"x(?:[0-9a-f]{2})*", value) for value in values)
                    assert all(value == "NA" or "\0" not in bytes.fromhex(value[1:]).decode() for value in values)
                case Storage.LOGICAL:
                    assert all(value in ("T", "F", "NA") for value in values)
                case _:
                    assert_never(storage)
            _ = literal(fingerprint)
        case _:
            assert_never(storage)
    return Case(expected_id, Lane.REPRESENTED, source, storage.value, fingerprint, error, warnings, stdout)


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == HEADER and len(lines) == 70
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {"integer": 9, "character": 38, "logical": 2, "error": 20}
    assert sum(bool(row.warnings) for row in rows) == 7
    assert all(not row.stdout for row in rows if row.warnings), "ordered effects plus warnings need explicit events"
    return rows


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    _ = read_cases()
    for storage, fingerprint in (
        ("integer", "integer|3|1,NA,-7"),
        ("integer", "integer|0|"),
        ("character", "character|3|x,NA,xf09f9880"),
        ("logical", "logical|3|T,F,NA"),
    ):
        assert parse_row(1, f"1\tx\t{storage}\t{fingerprint}\t-\t-\t-").fingerprint == fingerprint
    warning_hex = b"Warning: first\ncontinuation\nWarning: second\n".hex()
    error = parse_row(1, f"1\tx\terror\t-\t626f6f6d\t{warning_hex}\t41420a")
    assert (error.error, error.warnings, error.stdout) == ("boom", ("first\ncontinuation", "second"), "AB\n")
    for invalid in (
        "1\tx\tinteger\tinteger|1|1\t-\t-\t-\textra",
        "01\tx\tinteger\tinteger|1|1\t-\t-\t-",
        "2\tx\tinteger\tinteger|1|1\t-\t-\t-",
        "1\t\tinteger\tinteger|1|1\t-\t-\t-",
        "1\tx\tdouble\tdouble|0|\t-\t-\t-",
        "1\tx\tinteger\tlogical|1|T\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|1\t626f6f6d\t-\t-",
        "1\tx\tinteger\tinteger|01|1\t-\t-\t-",
        "1\tx\tinteger\tinteger|-1|\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|\t-\t-\t-",
        "1\tx\tinteger\tinteger|0|1\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|01\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|NaN\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|2147483648\t-\t-\t-",
        "1\tx\tinteger\tinteger|1|-2147483648\t-\t-\t-",
        "1\tx\tcharacter\tcharacter|1|x0\t-\t-\t-",
        "1\tx\tcharacter\tcharacter|1|xFF\t-\t-\t-",
        "1\tx\tcharacter\tcharacter|1|xff\t-\t-\t-",
        "1\tx\tcharacter\tcharacter|1|x00\t-\t-\t-",
        "1\tx\tlogical\tlogical|1|TRUE\t-\t-\t-",
        "1\tx\terror\tinteger|1|1\t626f6f6d\t-\t-",
        "1\tx\terror\t-\t-\t-\t-",
        "1\tx\terror\t-\t\t-\t-",
        "1\tx\terror\t-\tgg\t-\t-",
        "1\tx\terror\t-\tff\t-\t-",
        "1\tx\terror\t-\t626f6f6d\t6261640a\t-",
        "1\tx\terror\t-\t626f6f6d\t5761726e696e673a200a\t-",
        "1\tx\terror\t-\t626f6f6d\t5761726e696e673a20626164\t-",
        "1\tx\terror\t-\t626f6f6d\t-\t0",
        "1\tx\terror\t-\t626f6f6d\t-\t-\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)


if __name__ == "__main__":
    self_check()
    print("Unicode reader and shared comparator self-checks passed; 69 fixture rows validated")
