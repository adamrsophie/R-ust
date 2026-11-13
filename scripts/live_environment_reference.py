#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/live_environment_reference.py
from __future__ import annotations

import re
import sys
from dataclasses import replace
from enum import StrEnum
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Final, assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex, matches
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest, invocation, run

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/live-environments-r-4.6.0.tsv"
FIXTURE_SHA: Final = "a036503a72a0ed6b2b7ebf54bae5232e66ca3b4c2bd69eb967858925b45eeb2e"
HEADER: Final = "# R 4.6.0, JIT disabled, withVisible block evaluated directly at top level."
NAMES: Final = (
    "sibling",
    "counter",
    "late_global",
    "late_local",
    "local_force",
    "late_default",
    "default_local",
    "supplied_global",
    "forwarded",
    "paired",
    "independent",
    "lexical",
    "nearest",
    "reentrant",
    "recursive",
    "function_mode",
    "identity",
    "late_promise",
    "same_environment",
    "global_fallback",
)


class Storage(StrEnum):
    INTEGER = "integer"
    LOGICAL = "logical"


def parse_row(identifier: int, line: str) -> Case:
    name, source, kind, payload, visible, stdout_hex = line.split("\t")
    assert 1 <= identifier <= len(NAMES) and name == NAMES[identifier - 1] and source
    assert not any(character in line for character in "\r\n")
    assert visible == "TRUE"
    storage = Storage(kind)
    values = tuple(payload.split(","))
    match storage:
        case Storage.INTEGER:
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
        case Storage.LOGICAL:
            assert all(value in ("TRUE", "FALSE", "NA") for value in values)
            values = tuple({"TRUE": "T", "FALSE": "F", "NA": "NA"}[value] for value in values)
        case _:
            assert_never(storage)
    fingerprint = f"{storage.value}|{len(values)}|{','.join(values)}"
    _ = literal(fingerprint)
    return Case(identifier, Lane.REPRESENTED, source, kind, fingerprint, "", (), decode_hex(stdout_hex))


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == HEADER and len(lines) == len(NAMES) + 1
    return tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))


def program(row: Case) -> str:
    return (
        "options(warn=1);.live_result<-withVisible({"
        + row.source
        + "});cat(identical(.live_result$value,"
        + literal(row.fingerprint)
        + ')&&identical(.live_result$visible,TRUE),"\\n",sep="")\n'
    )


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    _ = read_cases()
    assert parse_row(1, "sibling\tx\tlogical\tTRUE,FALSE,NA\tTRUE\t31").fingerprint == "logical|3|T,F,NA"
    assert parse_row(2, "counter\tx\tinteger\t1,NA,-7\tTRUE\t-").fingerprint == "integer|3|1,NA,-7"
    for invalid in (
        "sibling\tx\tinteger\t1\tTRUE\t-\textra",
        "counter\tx\tinteger\t1\tTRUE\t-",
        "unknown\tx\tinteger\t1\tTRUE\t-",
        "sibling\t\tinteger\t1\tTRUE\t-",
        "sibling\tx\tdouble\t1\tTRUE\t-",
        "sibling\tx\tinteger\t\tTRUE\t-",
        "sibling\tx\tinteger\t01\tTRUE\t-",
        "sibling\tx\tinteger\tNaN\tTRUE\t-",
        "sibling\tx\tinteger\t2147483648\tTRUE\t-",
        "sibling\tx\tinteger\t-2147483648\tTRUE\t-",
        "sibling\tx\tlogical\tT\tTRUE\t-",
        "sibling\tx\tlogical\t1\tTRUE\t-",
        "sibling\tx\tinteger\t1\tFALSE\t-",
        "sibling\tx\tinteger\t1\tNA\t-",
        "sibling\tx\tinteger\t1\t1\t-",
        "sibling\tx\tinteger\t1\tTRUE\t",
        "sibling\tx\tinteger\t1\tTRUE\t0",
        "sibling\tx\tinteger\t1\tTRUE\tgg",
        "sibling\tx\tinteger\t1\tTRUE\tFF",
        "sibling\tx\tinteger\t1\tTRUE\tff",
        "sibling\tx\tinteger\t1\tTRUE\t-\n",
    ):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    expected = parse_row(1, 'sibling\tcat("AB");1L\tinteger\t1\tTRUE\t4142')
    with TemporaryDirectory(prefix="live-environment-self-check-") as directory:
        source_file = Path(directory) / "program.R"
        for source, passed in (
            (expected.source, True),
            ('cat("AB");2L', False),
            ('cat("AB");invisible(1L)', False),
            ('cat("BA");1L', False),
            ('cat("AB");1', False),
        ):
            _ = source_file.write_text(program(replace(expected, source=source)))
            separate = run(invocation("rscript-file", ("R", "Rscript"), source_file))
            merged = run(invocation("rscript-file", ("R", "Rscript"), source_file), merged=True)
            assert matches(expected, (separate, merged), reference=True) is passed, (source, separate, merged)


if __name__ == "__main__":
    self_check()
    print("Live-environment reader, typed value, visibility and output checks passed; 20 rows validated")
