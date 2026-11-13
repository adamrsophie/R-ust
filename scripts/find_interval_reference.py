#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/find_interval_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import replace
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, matches
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

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/find-interval-r-4.6.0.tsv"
FIXTURE_SHA: Final = "486bca157931af07de4f6d1494a843d50012eee4d9e9ad7bc15f773185e48301"
HEADER: Final = "# R version 4.6.0 (2026-04-24) findInterval C locale"
COLUMNS: Final = "id\tsource\tkind\tdata"


class Storage(StrEnum):
    INTEGER = "integer"
    LOGICAL = "logical"
    ERROR = "error"


def parse_row(expected_id: int, line: str) -> Case:
    identifier, source, storage_text, payload = line.split("\t")
    assert identifier == str(expected_id) and source
    assert not any(character in line for character in "\r\n")
    storage = Storage(storage_text)
    error = ""
    match storage:
        case Storage.ERROR:
            assert payload and payload != "-"
            fingerprint = "-"
            error = payload
        case Storage.INTEGER:
            values = () if payload == "-" else tuple(payload.split(","))
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
            fingerprint = f"{storage.value}|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
        case Storage.LOGICAL:
            values = () if payload == "-" else tuple(payload.split(","))
            assert all(value in ("TRUE", "FALSE", "NA") for value in values)
            values = tuple({"TRUE": "T", "FALSE": "F", "NA": "NA"}[value] for value in values)
            fingerprint = f"{storage.value}|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
        case _:
            assert_never(storage)
    return Case(expected_id, Lane.REPRESENTED, source, storage.value, fingerprint, error, (), "")


def parse_fixture(text: str) -> tuple[Case, ...]:
    lines = text.splitlines()
    assert len(lines) == 129 and lines[0] == HEADER and lines[1] == COLUMNS
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[2:], 1))
    assert Counter(row.storage for row in rows) == {"integer": 76, "error": 39, "logical": 12}
    assert all("findInterval(" in row.source for row in rows[:107])
    assert all(row.source.startswith("is.unsorted(") for row in rows[107:115])
    assert all(" && " in row.source or " || " in row.source for row in rows[115:])
    return rows


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    return parse_fixture(FIXTURE.read_text())


def sample_run(stdout: str, stderr: str, status: int) -> Run:
    return Run(
        Invocation(("fixture-self-check",), b""),
        status,
        stdout,
        stderr,
        encode_bytes(stdout.encode()),
        encode_bytes(stderr.encode()),
    )


def fixture_comparator_self_check(rows: tuple[Case, ...]) -> None:
    for row in rows:
        for reference in (False, True):
            output = "" if row.error else "TRUE\n"
            error = "Error: " + row.error + "\n" if row.error else ""
            footer = "Execution halted\n" if row.error and reference else ""
            separate = sample_run(output, error + footer, int(bool(row.error)))
            merged = sample_run(output + error + footer, "", separate.status)
            assert matches(row, (separate, merged), reference=reference)
            mutations = (
                (replace(separate, status=1 - separate.status), merged),
                (separate, replace(merged, status=1 - merged.status)),
                (replace(separate, stdout=separate.stdout + "AFTER\n"), merged),
                (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
                (separate, replace(merged, stderr="unexpected\n")),
                (replace(separate, stderr="Warning: unexpected\n" + separate.stderr), merged),
                (separate, replace(merged, stdout="Warning: unexpected\n" + merged.stdout)),
                (replace(separate, stderr=separate.stderr + "changed"), merged),
                (separate, replace(merged, stdout=merged.stdout + "changed")),
            )
            for changed in mutations:
                assert not matches(row, changed, reference=reference), (row.identifier, changed)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    fixture_comparator_self_check(read_cases())
    value = parse_row(1, "1\tfindInterval(1,1)\tinteger\t1,NA,-7")
    assert value.fingerprint == "integer|3|1,NA,-7"
    assert literal(value.fingerprint) == "c(1L,NA_integer_,-7L)"
    assert parse_row(1, "1\tfindInterval(NULL,1)\tinteger\t-").fingerprint == "integer|0|"
    logical = parse_row(1, "1\tx\tlogical\tTRUE,FALSE,NA")
    assert logical.fingerprint == "logical|3|T,F,NA"
    assert literal(logical.fingerprint) == "c(TRUE,FALSE,NA)"
    assert parse_row(1, "1\tx\terror\tboom").error == "boom"
    for invalid in (
        "1\tx\tinteger",
        "1\tx\tinteger\t1\textra",
        "2\tx\tinteger\t1",
        "01\tx\tinteger\t1",
        "1\t\tinteger\t1",
        "1\tx\tdouble\t1",
        "1\tx\tinteger\t",
        "1\tx\tinteger\tNaN",
        "1\tx\tinteger\t01",
        "1\tx\tinteger\t+1",
        "1\tx\tinteger\t-0",
        "1\tx\tinteger\t1,",
        "1\tx\tinteger\t2147483648",
        "1\tx\tinteger\t-2147483648",
        "1\tx\tlogical\t",
        "1\tx\tlogical\ttrue",
        "1\tx\tlogical\t1",
        "1\tx\tlogical\tTRUE,",
        "1\tx\terror\t-",
        "1\tx\terror\t",
        "1\tx\terror\tboom\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    fixture = FIXTURE.read_text()
    for invalid_fixture in (
        "",
        fixture.replace(HEADER, "# invalid header", 1),
        fixture.replace(COLUMNS, "id\tsource\tkind\tvalue", 1),
        "\n".join(fixture.splitlines()[:-1]),
        fixture + fixture.splitlines()[-1] + "\n",
        fixture.replace("\n2\t", "\n1\t", 1),
    ):
        try:
            _ = parse_fixture(invalid_fixture)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")


if __name__ == "__main__":
    self_check()
    print("findInterval reader and shared comparator self-checks passed; 127 fixture rows validated")
