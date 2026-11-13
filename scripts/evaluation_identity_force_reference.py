#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/evaluation_identity_force_reference.py
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Case, Lane, decode_hex
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/evaluation-identity-force-r-4.6.0.tsv"
FIXTURE_SHA: Final = "98c560ea77d8a7e08a86a9cb37563e5cbd08a1b2d7008ec0ad85629548fa24fc"
HEADER: Final = "owner\tid\tsource\ttype\tvalue\tvisible\tstdout_hex\twarnings_hex"
NAMES: Final = (
    "integer",
    "named",
    "null",
    "empty",
    "logical",
    "string",
    "integers",
    "real_storage",
    "list_value",
    "matrix_value",
    "missing",
    "explicit_missing",
    "blank",
    "duplicate",
    "unused",
    "too_many",
    "overlong_not_partial",
    "exact_before_positional",
    "supplied_effect",
    "stop_effect",
    "unforced",
    "forced_once",
    "default",
    "default_dependency",
    "unused_default",
    "dots",
    "dots_named",
    "dots_unused",
    "dots_duplicate",
    "alias",
    "parameter_alias",
    "shadow",
    "shadow_non_function",
    "caller_x",
    "caller_binding",
    "invisible",
    "assignment_visibility",
    "nested_visibility",
    "repeated_visibility",
    "return_value",
    "return_invisible",
    "return_effect",
    "top_return",
    "top_break",
    "top_next",
    "loop_break",
    "loop_next",
    "loop_return",
    "closure_retained",
    "closure_lazy_after",
    "changed_binding",
    "closure_storage",
)


@dataclass(frozen=True, slots=True)
class EvaluationCase(Case):
    owner: str
    name: str
    visible: str


def parse_row(identifier: int, line: str) -> EvaluationCase:
    assert 1 <= identifier <= 104 and not any(character in line for character in "\r\n")
    owner, name, source, storage, payload, visible, stdout_hex, warnings_hex = line.split("\t")
    assert owner == ("identity" if identifier <= 52 else "force")
    assert name == NAMES[(identifier - 1) % 52] and source and owner in source
    assert storage in ("integer", "logical", "character", "NULL", "error")
    assert warnings_hex == "-"
    stdout = decode_hex(stdout_hex)
    error = ""
    if storage == "error":
        error = decode_hex(payload)
        assert error and visible == "-"
        fingerprint = "-"
    else:
        assert visible in ("TRUE", "FALSE")
        kind, count_text, data = payload.split("|", 2)
        assert kind == storage and re.fullmatch(r"0|[1-9][0-9]*", count_text)
        values = () if not data else tuple(data.split(","))
        assert len(values) == int(count_text)
        if kind == "NULL":
            assert payload == "NULL|0|"
        elif kind == "integer":
            assert all(re.fullmatch(r"NA|0|-?[1-9][0-9]*", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
        elif kind == "logical":
            assert all(value in ("T", "F", "NA") for value in values)
        else:
            assert all(value == "NA" or re.fullmatch(r"x(?:[0-9a-f]{2})*", value) for value in values)
            for value in values:
                if value != "NA":
                    _ = bytes.fromhex(value[1:]).decode()
        fingerprint = "NULL" if kind == "NULL" else payload
        _ = literal(fingerprint)
    return EvaluationCase(
        identifier, Lane.REPRESENTED, source, storage, fingerprint, error, (), stdout, owner, name, visible
    )


def parse_fixture(text: str) -> tuple[EvaluationCase, ...]:
    assert "\r" not in text and text.endswith("\n")
    lines = text.splitlines()
    assert len(lines) == 105 and lines[0] == HEADER and len(NAMES) == len(set(NAMES)) == 52
    rows = tuple(parse_row(identifier, line) for identifier, line in enumerate(lines[1:], 1))
    assert Counter(row.storage for row in rows) == {"integer": 62, "logical": 8, "character": 4, "NULL": 2, "error": 28}
    assert Counter(row.visible for row in rows) == {"TRUE": 68, "FALSE": 8, "-": 28}
    assert sum(bool(row.stdout) for row in rows) == 20
    return rows


def read_cases() -> tuple[EvaluationCase, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    return parse_fixture(FIXTURE.read_text())


def program(row: EvaluationCase) -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    return (
        "options(warn=1);.evaluation_answer<-withVisible({"
        + row.source
        + "});cat(identical(.evaluation_answer$value,"
        + literal(row.fingerprint)
        + ") && identical(.evaluation_answer$visible,"
        + row.visible
        + '),"\\n",sep="")\n'
    )


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    rows = read_cases()
    for row in rows:
        source = program(row)
        assert source.count(row.source) == 1
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
        if not row.error:
            assert f"identical(.evaluation_answer$value,{literal(row.fingerprint)})" in source
            assert f"&& identical(.evaluation_answer$visible,{row.visible})" in source
            assert "withVisible({" + row.source + "})" in source
    fixture = FIXTURE.read_text()
    lines = fixture.splitlines()
    for payload in ("integer|0|", "integer|3|-2147483647,NA,2147483647"):
        row = parse_row(1, f"identity\tinteger\tidentity(3L)\tinteger\t{payload}\tTRUE\t-\t-")
        assert row.fingerprint == payload
    for payload in ("character|3|x,x4e41,NA", "character|1|xc3a9"):
        row = parse_row(6, f"identity\tstring\tidentity('x')\tcharacter\t{payload}\tFALSE\t-\t-")
        assert row.fingerprint == payload
    for column, replacement in (
        (0, "unknown"),
        (0, "force"),
        (1, "missing"),
        (2, ""),
        (3, "raw"),
        (3, "logical"),
        (4, "integer|01|3"),
        (4, "integer|-1|"),
        (4, "integer|2|3"),
        (4, "integer|1|"),
        (4, "integer|1|01"),
        (4, "integer|1|+1"),
        (4, "integer|1|-0"),
        (4, "integer|1|2147483648"),
        (4, "integer|1|-2147483648"),
        (5, "-"),
        (5, "true"),
        (6, ""),
        (6, "4A"),
        (6, "0"),
        (6, "ff"),
        (7, "61"),
    ):
        fields = lines[1].split("\t")
        fields[column] = replacement
        try:
            _ = parse_row(1, "\t".join(fields))
        except (AssertionError, ValueError):
            continue
        raise AssertionError((column, replacement))
    for index, column, replacement in (
        (3, 4, "NULL|1|x"),
        (5, 4, "logical|1|TRUE"),
        (6, 4, "character|1|x0"),
        (6, 4, "character|1|xFF"),
        (6, 4, "character|1|xff"),
        (6, 4, "character|1|bad"),
        (11, 4, "-"),
        (11, 4, "ff"),
        (11, 4, "AA"),
        (11, 5, "TRUE"),
    ):
        fields = lines[index].split("\t")
        fields[column] = replacement
        try:
            _ = parse_row(index, "\t".join(fields))
        except (AssertionError, ValueError):
            continue
        raise AssertionError((index, column, replacement))
    for invalid in (
        "",
        fixture.rstrip("\n"),
        fixture.replace(HEADER, "invalid", 1),
        fixture + lines[1] + "\n",
        "\n".join(lines[:-1]) + "\n",
        fixture.replace("\n", "\r\n"),
        fixture.replace(lines[2], lines[1], 1),
        fixture.replace("\tTRUE\t", "\tFALSE\t", 1),
        fixture.replace("\t41\t", "\t-\t", 1),
    ):
        try:
            _ = parse_fixture(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")
    for invalid in (lines[1] + "\textra", lines[1] + "\n", "\t".join(lines[1].split("\t")[:-1])):
        try:
            _ = parse_row(1, invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid row accepted")


if __name__ == "__main__":
    self_check()
    print("Identity/force reader and shared comparator self-checks passed; 104 supported source rows")
