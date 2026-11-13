#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/encode_string_reference.py
from __future__ import annotations

import sys
from collections import Counter
from dataclasses import replace
from pathlib import Path
from typing import Final

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

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/encode-string-r-4.6.0.tsv"
FIXTURE_SHA: Final = "df23c35d74034b6e380d8c3c3bd258f8fdf9e6b855675f7e8e7eef34e8b33ee7"
HEADER: Final = "# R 4.6.0 en_US.UTF-8 encodeString v1"


def parse_row(expected_id: int, line: str) -> Case:
    identifier, source, payload, error_hex, warnings_hex, stdout_hex = line.split("\t")
    assert identifier == str(expected_id) and source
    assert not any(character in line for character in "\r\n")
    error = decode_hex(error_hex)
    warnings = tuple(decode_hex(warnings_hex).split("\n")) if warnings_hex != "-" else ()
    stdout = decode_hex(stdout_hex)
    assert all(warnings)
    assert not (warnings and stdout), "ordered effects plus warnings need explicit events"
    if error:
        assert payload == "-"
        storage = "error"
        fingerprint = "-"
    else:
        storage = "character"
        values = () if payload == "EMPTY" else tuple(payload.split(","))
        for value in values:
            if value != "NA":
                assert "\0" not in decode_hex(value)
        encoded = tuple("NA" if value == "NA" else "x" + ("" if value == "-" else value) for value in values)
        fingerprint = f"character|{len(encoded)}|{','.join(encoded)}"
        _ = literal(fingerprint)
    return Case(expected_id, Lane.REPRESENTED, source, storage, fingerprint, error, warnings, stdout)


def parse_fixture(text: str) -> tuple[Case, ...]:
    lines = text.splitlines()
    assert len(lines) == 61 and lines[0] == HEADER
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {"character": 40, "error": 20}
    assert sum(bool(row.warnings) for row in rows) == 4
    assert [(row.identifier, row.stdout) for row in rows if row.stdout] == [
        (53, "x\njustify\nwidth\nquote\nna\n"),
        (54, "x\n"),
        (56, "quote\nna\n"),
    ]
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
                if row.stdout:
                    mutations.append((separate, replace(merged, stdout=warnings + error + footer + output)))
            if row.warnings:
                mutations.append((replace(separate, stderr=error + footer), merged))
                mutations.append((separate, replace(merged, stdout=output + error + footer + warnings)))
            for changed in mutations:
                assert not matches(row, changed, reference=reference), (row.identifier, changed)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    fixture_comparator_self_check(read_cases())
    value = parse_row(1, "1\tx\t61,-,NA,f09f9982\t-\t-\t780a")
    assert value.fingerprint == "character|4|x61,x,NA,xf09f9982" and value.stdout == "x\n"
    assert literal(value.fingerprint) == 'c("a","",NA_character_,"\\ud83d\\ude42")'
    assert parse_row(1, "1\tx\tEMPTY\t-\t-\t-").fingerprint == "character|0|"
    assert parse_row(1, "1\tx\t-\t-\t-\t-").fingerprint == "character|1|x"
    error = parse_row(1, "1\tx\t-\t626f6f6d\t66697273740a7365636f6e64\t-")
    assert error.error == "boom" and error.warnings == ("first", "second")
    for invalid in (
        "1\tx\t61\t-\t-",
        "1\tx\t61\t-\t-\t-\textra",
        "2\tx\t61\t-\t-\t-",
        "01\tx\t61\t-\t-\t-",
        "1\t\t61\t-\t-\t-",
        "1\tx\t61\t626f6f6d\t-\t-",
        "1\tx\tEMPTY\t626f6f6d\t-\t-",
        "1\tx\t\t-\t-\t-",
        "1\tx\tNA,\t-\t-\t-",
        "1\tx\tNA,EMPTY\t-\t-\t-",
        "1\tx\tNaN\t-\t-\t-",
        "1\tx\t6\t-\t-\t-",
        "1\tx\t6A\t-\t-\t-",
        "1\tx\tff\t-\t-\t-",
        "1\tx\t00\t-\t-\t-",
        "1\tx\t-\t\t-\t-",
        "1\tx\t-\tBOOM\t-\t-",
        "1\tx\t-\tff\t-\t-",
        "1\tx\t-\t626f6f6d\t66697273740a\t-",
        "1\tx\t-\t626f6f6d\t0a6669727374\t-",
        "1\tx\t-\t626f6f6d\t\t-",
        "1\tx\t-\t626f6f6d\tff\t-",
        "1\tx\t-\t626f6f6d\t-\t5A",
        "1\tx\t-\t626f6f6d\t-\tff",
        "1\tx\t-\t626f6f6d\t6669727374\t780a",
        "1\tx\t-\t626f6f6d\t-\t-\n",
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
        fixture.replace("\t780a\n", "\t780a780a\n", 1),
    ):
        try:
            _ = parse_fixture(invalid_fixture)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")


if __name__ == "__main__":
    self_check()
    print("encodeString reader and shared comparator self-checks passed; 60 fixture rows validated")
