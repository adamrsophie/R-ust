#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/is_element_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import replace
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex, program
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.is_element_capture import LIMITS, limitation
from scripts.is_element_capture import self_check as capture_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/is-element-r-4.6.0.tsv"
FIXTURE_SHA: Final = "260291a4e877e9534631f285f5480fd08462fe93425697155b51936ebb0320f9"
MATCH_FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/match-coercion-r-4.6.0.tsv"
MATCH_SHA: Final = "4b2ee76bdaffe1cb5b8a23e20c2d99f3164e94ebf273837da440187ed3c76708"
HEADER: Final = "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"


class Storage(StrEnum):
    INTEGER = "integer"
    LOGICAL = "logical"
    ERROR = "error"


def parse_row(expected_id: int, line: str, *, shared: bool = False) -> Case:
    assert not any(character in line for character in "\r\n")
    identifier, name, source, storage_text, fingerprint, error_hex, warnings_hex, stdout_hex = line.split("\t")
    storage = Storage(storage_text)
    assert identifier == str(expected_id) and source and re.fullmatch(r"[a-z_]+", name)
    assert storage in (Storage.INTEGER, Storage.LOGICAL) if shared else storage in (Storage.LOGICAL, Storage.ERROR)
    error = decode_hex(error_hex)
    warnings = tuple(decode_hex(warnings_hex).split("\n")) if warnings_hex != "-" else ()
    stdout = decode_hex(stdout_hex)
    assert all(warnings) and not (stdout and warnings)
    match storage:
        case Storage.ERROR:
            assert error and fingerprint == "-"
        case Storage.INTEGER | Storage.LOGICAL:
            assert not error
            kind, count_text, payload = fingerprint.split("|", 2)
            assert kind == storage.value and re.fullmatch(r"0|[1-9][0-9]*", count_text)
            values = () if not payload else tuple(payload.split(","))
            assert len(values) == int(count_text)
            match storage:
                case Storage.INTEGER:
                    assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
                    assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
                case Storage.LOGICAL:
                    assert all(value in ("T", "F", "NA") for value in values)
                case _:
                    assert_never(storage)
            _ = literal(fingerprint)
        case _:
            assert_never(storage)
    row = Case(
        40 + expected_id if shared else expected_id,
        Lane.REPRESENTED,
        source,
        storage.value,
        fingerprint,
        error,
        warnings,
        stdout,
    )
    if limitation(row) is not None:
        row = replace(row, lane=Lane.FUTURE)
    return row


def parse_fixtures(text: str, shared_text: str) -> tuple[Case, ...]:
    lines, shared_lines = text.splitlines(), shared_text.splitlines()
    assert len(lines) == 41 and lines[0] == HEADER
    assert len(shared_lines) == 21 and shared_lines[0] == HEADER
    wrapper = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    shared = tuple(parse_row(identifier, line, shared=True) for identifier, line in enumerate(shared_lines[1:], 1))
    assert Counter(row.storage for row in wrapper) == {"logical": 31, "error": 9}
    assert Counter(row.storage for row in shared) == {"integer": 12, "logical": 8}
    assert [row.identifier for row in wrapper if row.warnings] == [36]
    assert [(row.identifier, row.stdout) for row in wrapper if row.stdout] == [(26, "ELSET"), (27, "1"), (28, "SET")]
    assert all(not row.warnings and not row.stdout and not row.error for row in shared)
    rows = wrapper + shared
    assert tuple(row.identifier for row in rows) == tuple(range(1, 61))
    assert tuple(limitation(row) for row in rows if limitation(row) is not None) == LIMITS
    assert rows[26].lane == Lane.REPRESENTED
    return rows


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA and digest(MATCH_FIXTURE) == MATCH_SHA
    return parse_fixtures(FIXTURE.read_text(), MATCH_FIXTURE.read_text())


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    capture_self_check(rows)
    for row in rows:
        source = program(row)
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
    assert parse_row(1, "1\tx\tx\tlogical\tlogical|0|\t-\t-\t-").fingerprint == "logical|0|"
    assert parse_row(1, "1\tx\tx\tinteger\tinteger|3|1,NA,-7\t-\t-\t-", shared=True).fingerprint == "integer|3|1,NA,-7"
    for invalid in (
        "1\tx\tx\tlogical\tlogical|1|T\t-\t-",
        "1\tx\tx\tlogical\tlogical|1|T\t-\t-\t-\textra",
        "2\tx\tx\tlogical\tlogical|1|T\t-\t-\t-",
        "01\tx\tx\tlogical\tlogical|1|T\t-\t-\t-",
        "1\t\tx\tlogical\tlogical|1|T\t-\t-\t-",
        "1\tx\t\tlogical\tlogical|1|T\t-\t-\t-",
        "1\tx\tx\tdouble\tdouble|0|\t-\t-\t-",
        "1\tx\tx\tlogical\tinteger|1|1\t-\t-\t-",
        "1\tx\tx\tlogical\tlogical|01|T\t-\t-\t-",
        "1\tx\tx\tlogical\tlogical|2|T\t-\t-\t-",
        "1\tx\tx\tlogical\tlogical|1|TRUE\t-\t-\t-",
        "1\tx\tx\tlogical\tlogical|1|T\t626f6f6d\t-\t-",
        "1\tx\tx\tlogical\tlogical|1|T\t-\t61\t62",
        "1\tx\tx\terror\tlogical|1|T\t626f6f6d\t-\t-",
        "1\tx\tx\terror\t-\t-\t-\t-",
        "1\tx\tx\terror\t-\tff\t-\t-",
        "1\tx\tx\terror\t-\t\t-\t-",
        "1\tx\tx\terror\t-\t626f6f6d\t66697273740a\t-",
        "1\tx\tx\terror\t-\t626f6f6d\t-\t5A",
        "1\tx\tx\terror\t-\t626f6f6d\t-\tff",
        "1\tx\tx\terror\t-\t626f6f6d\t-\t-\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    for payload in (
        "integer|1|2147483648",
        "integer|1|-2147483648",
        "integer|1|01",
        "integer|1|+1",
        "integer|1|-0",
        "integer|1|NaN",
        "integer|2|1",
    ):
        try:
            _ = parse_row(1, f"1\tx\tx\tinteger\t{payload}\t-\t-\t-", shared=True)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(payload)
    fixture, shared_fixture = FIXTURE.read_text(), MATCH_FIXTURE.read_text()
    for invalid, invalid_shared in (
        ("", shared_fixture),
        (fixture, ""),
        (fixture.replace(HEADER, "invalid", 1), shared_fixture),
        (fixture, shared_fixture.replace(HEADER, "invalid", 1)),
        ("\n".join(fixture.splitlines()[:-1]), shared_fixture),
        (fixture, "\n".join(shared_fixture.splitlines()[:-1])),
        (fixture.replace("\n2\t", "\n1\t", 1), shared_fixture),
        (fixture, shared_fixture.replace("\n2\t", "\n1\t", 1)),
        (fixture.replace(LIMITS[0].source, "is.element(1L,1L)", 1), shared_fixture),
    ):
        try:
            _ = parse_fixtures(invalid, invalid_shared)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixtures accepted")


if __name__ == "__main__":
    self_check()
    print("is.element/shared matching readers passed; 60 sources, 57 supported including row27, 3 LIMIT")
