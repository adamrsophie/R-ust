#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/strtoi_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import replace
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex, matches
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

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/strtoi-r-4.6.0.tsv"
FIXTURE_SHA: Final = "bfeffe59cb673041d5396cd1efbc35bf9f5a8e8d80b65c25d3e35bf574893bcf"
HEADER: Final = "# R version 4.6.0 (2026-04-24) STRTOI UTF-8 en_US.UTF-8"


class Storage(StrEnum):
    INTEGER = "integer"
    ERROR = "error"


def parse_row(expected_id: int, line: str) -> Case:
    identifier, source, storage_text, payload, error_hex, warnings_hex, stdout_hex = line.split("\t")
    assert identifier == str(expected_id) and source
    assert not any(character in line for character in "\r\n")
    error = decode_hex(error_hex)
    warnings = tuple(decode_hex(warnings_hex).split("\n")) if warnings_hex != "-" else ()
    stdout = decode_hex(stdout_hex)
    assert all(warnings)
    assert not (warnings and stdout), "ordered effects plus warnings need explicit events"
    storage = Storage(storage_text)
    match storage:
        case Storage.ERROR:
            assert error and payload == "-"
            fingerprint = "-"
        case Storage.INTEGER:
            assert not error
            values = () if payload == "-" else tuple(payload.split(","))
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
            fingerprint = f"integer|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
        case _:
            assert_never(storage)
    return Case(expected_id, Lane.REPRESENTED, source, storage.value, fingerprint, error, warnings, stdout)


def parse_fixture(text: str) -> tuple[Case, ...]:
    lines = text.splitlines()
    assert len(lines) == 124 and lines[0] == HEADER
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {"integer": 82, "error": 41}
    assert sum(bool(row.warnings) for row in rows) == 12
    assert [(row.identifier, row.stdout) for row in rows if row.stdout] == [(20, "XB"), (33, "XBE")]
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
            output = row.stdout + ("" if row.error else "TRUE\n")
            warnings = "".join("Warning: " + text + "\n" for text in row.warnings)
            error = "Error: " + row.error + "\n" if row.error else ""
            footer = "Execution halted\n" if row.error and reference else ""
            separate = sample_run(output, warnings + error + footer, int(bool(row.error)))
            merged = sample_run(warnings + output + error + footer, "", separate.status)
            assert matches(row, (separate, merged), reference=reference)
            mutations = [
                (replace(separate, status=1 - separate.status), merged),
                (separate, replace(merged, status=1 - merged.status)),
                (replace(separate, stdout=separate.stdout + "AFTER\n"), merged),
                (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
                (separate, replace(merged, stderr="unexpected\n")),
            ]
            if row.stdout:
                mutations.append((replace(separate, stdout=output.replace(row.stdout, row.stdout[::-1])), merged))
                mutations.append((separate, replace(merged, stdout=merged.stdout.replace(row.stdout, ""))))
            if row.error:
                mutations.append(
                    (replace(separate, stderr=warnings + error.replace(row.error, "changed") + footer), merged)
                )
                mutations.append((separate, replace(merged, stdout=warnings + error + footer + output)))
            if row.warnings:
                mutations.append((replace(separate, stderr=error + footer), merged))
                mutations.append((separate, replace(merged, stdout=output + error + footer + warnings)))
            for changed in mutations:
                if changed == (separate, merged):
                    continue
                assert not matches(row, changed, reference=reference), (row.identifier, changed)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    fixture_comparator_self_check(read_cases())
    value = parse_row(1, '1\tstrtoi("1")\tinteger\t1,NA,-7\t-\t-\t5842')
    assert value.fingerprint == "integer|3|1,NA,-7" and value.stdout == "XB"
    assert literal(value.fingerprint) == "c(1L,NA_integer_,-7L)"
    assert parse_row(1, "1\tstrtoi(NULL)\tinteger\t-\t-\t-\t-").fingerprint == "integer|0|"
    error = parse_row(1, "1\tx\terror\t-\t626f6f6d\t66697273740a7365636f6e64\t-")
    assert error.error == "boom" and error.warnings == ("first", "second")
    assert parse_row(1, "1\tx\terror\t-\t626f6f6d\t-\t584245").stdout == "XBE"
    for invalid in (
        "1\tx\tinteger\t1\t-\t-",
        "1\tx\tinteger\t1\t-\t-\t-\textra",
        "2\tx\tinteger\t1\t-\t-\t-",
        "01\tx\tinteger\t1\t-\t-\t-",
        "1\t\tinteger\t1\t-\t-\t-",
        "1\tx\tlogical\tTRUE\t-\t-\t-",
        "1\tx\tinteger\t1\t626f6f6d\t-\t-",
        "1\tx\tinteger\t\t-\t-\t-",
        "1\tx\tinteger\tNaN\t-\t-\t-",
        "1\tx\tinteger\t01\t-\t-\t-",
        "1\tx\tinteger\t+1\t-\t-\t-",
        "1\tx\tinteger\t-0\t-\t-\t-",
        "1\tx\tinteger\t1,\t-\t-\t-",
        "1\tx\tinteger\t2147483648\t-\t-\t-",
        "1\tx\tinteger\t-2147483648\t-\t-\t-",
        "1\tx\terror\t1\t626f6f6d\t-\t-",
        "1\tx\terror\t-\t-\t-\t-",
        "1\tx\terror\t-\t\t-\t-",
        "1\tx\terror\t-\tBOOM\t-\t-",
        "1\tx\terror\t-\t6\t-\t-",
        "1\tx\terror\t-\tff\t-\t-",
        "1\tx\terror\t-\t626f6f6d\t66697273740a\t-",
        "1\tx\terror\t-\t626f6f6d\t0a6669727374\t-",
        "1\tx\terror\t-\t626f6f6d\t\t-",
        "1\tx\terror\t-\t626f6f6d\tff\t-",
        "1\tx\terror\t-\t626f6f6d\t-\t5A",
        "1\tx\terror\t-\t626f6f6d\t-\tff",
        "1\tx\terror\t-\t626f6f6d\t6669727374\t5842",
        "1\tx\terror\t-\t626f6f6d\t-\t-\n",
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
        "\n".join(fixture.splitlines()[:-1]),
        fixture + fixture.splitlines()[-1] + "\n",
        fixture.replace("\n2\t", "\n1\t", 1),
        fixture.replace("\t5842\n", "\t4258\n", 1),
        fixture.replace("\t584245\n", "\t5842\n", 1),
    ):
        try:
            _ = parse_fixture(invalid_fixture)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")


if __name__ == "__main__":
    self_check()
    print("STRTOI reader and shared comparator self-checks passed; 123 fixture rows validated")
