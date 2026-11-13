#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/primitive_values_reference.py
from __future__ import annotations

import hashlib
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Final

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import ROOT, Lane, decode_hex
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.primitive_values_profile import (
    HANDLE_SOURCES,
    Handle,
    PrimitiveCase,
    program,
    read_metadata,
)
from scripts.primitive_values_profile import self_check as profile_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/primitive-values-r-4.6.0.tsv"
FIXTURE_SHA: Final = "a906a3975811ae4fb5110794c2d8cb380d20886b606539f7388a75f075834db5"
ORIGINAL_SOURCES_SHA: Final = "ebdffc9badaa13cd78f3320b4824555e341de2bbf83e268ad2decc214b967481"
EXPANDED_SOURCES_SHA: Final = "24d55e77111d25100c717667cb866ba6824838c151f565fc7e71717441ce2ba3"


def value_fingerprint(payload: str) -> tuple[str, str]:
    if payload == "NULL":
        return "NULL", "NULL"
    kind, values_text = payload.split(":", 1)
    values = tuple(values_text.split(","))
    assert kind in ("integer", "logical", "double", "character")
    if kind == "integer":
        assert all(re.fullmatch(r"NA|0|-?[1-9][0-9]*", value) for value in values)
        assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
    elif kind == "logical":
        assert all(value in ("TRUE", "FALSE", "NA") for value in values)
        values = tuple({"TRUE": "T", "FALSE": "F", "NA": "NA"}[value] for value in values)
    elif kind == "double":
        assert values == ("1",), "the immutable fixture represents only the exactly encoded double 1"
        values = ("3ff0000000000000",)
    else:
        assert len(values) == 1 and values[0] in ("builtin", "special", "function", "closure")
        values = ("x" + values[0].encode().hex(),)
    fingerprint = f"{kind}|{len(values)}|{','.join(values)}"
    _ = literal(fingerprint)
    return kind, fingerprint


def parse_row(identifier: int, line: str, metadata: tuple[Handle, ...]) -> PrimitiveCase:
    assert 1 <= identifier <= 95 and not any(character in line for character in "\r\n\0")
    source, status, payload, visible, stdout_hex = line.split("\t")
    assert source and payload and status in ("OK", "ERR")
    stdout = decode_hex(stdout_hex) if stdout_hex else ""
    assert stdout_hex != "-"
    handle = None
    error = ""
    display = ""
    if status == "ERR":
        assert visible == "-" and payload.strip() == payload
        storage, fingerprint, error = "error", "-", payload
    else:
        assert visible in ("TRUE", "FALSE")
        if payload.startswith(("builtin:", "special:")):
            assert identifier <= 5 and source == HANDLE_SOURCES[identifier - 1]
            handle = metadata[identifier - 1]
            assert handle.source == source and handle.visible == visible
            assert payload == handle.kind + ':.Primitive("' + source.strip("`") + '")'
            storage, fingerprint, display = handle.kind, payload, handle.display
        else:
            assert identifier > 5
            storage, fingerprint = value_fingerprint(payload)
    return PrimitiveCase(
        identifier, Lane.REPRESENTED, source, storage, fingerprint, error, (), stdout + display, visible, stdout, handle
    )


def parse_fixture(text: str, metadata: tuple[Handle, ...]) -> tuple[PrimitiveCase, ...]:
    assert "\r" not in text and text.endswith("\n") and len(metadata) == 5
    lines = text.splitlines()
    assert len(lines) == len(set(lines)) == 95
    rows = tuple(parse_row(identifier, line, metadata) for identifier, line in enumerate(lines, 1))
    assert Counter(row.storage for row in rows) == {
        "integer": 30,
        "logical": 24,
        "character": 9,
        "double": 1,
        "NULL": 1,
        "builtin": 3,
        "special": 2,
        "error": 25,
    }
    assert Counter(row.visible for row in rows) == {"TRUE": 64, "FALSE": 6, "-": 25}
    assert [row.identifier for row in rows if row.source_stdout] == [40, 41, 86, 87, 89, 90]
    assert tuple(row.source for row in rows[:5]) == HANDLE_SOURCES
    for selected, expected in ((rows[:72], ORIGINAL_SOURCES_SHA), (rows, EXPANDED_SOURCES_SHA)):
        assert hashlib.sha256("".join(row.source + "\n" for row in selected).encode()).hexdigest() == expected
    return rows


def read_cases() -> tuple[PrimitiveCase, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    return parse_fixture(FIXTURE.read_text(), read_metadata())


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
    profile_self_check()
    rows = read_cases()
    for row in rows:
        source = program(row)
        assert ('cat("AFTER\\n")' in source) == bool(row.error)
        if row.error:
            assert source == "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
        else:
            assert source.count("withVisible({" + row.source + "})") == 1
            assert f"&& identical(.primitive_answer$visible,{row.visible})" in source
            if row.handle is None:
                assert f"identical(.primitive_answer$value,{literal(row.fingerprint)})" in source
            else:
                assert "print(.primitive_answer$value);" in source
                assert f'identical(typeof(.primitive_answer$value),"{row.storage}")' in source
                assert 'identical(class(.primitive_answer$value),"function")' in source
                assert "identical(is.function(.primitive_answer$value),TRUE)" in source
                assert "identical(length(.primitive_answer$value),1L)" in source
    assert value_fingerprint("integer:-2147483647,NA,2147483647") == ("integer", "integer|3|-2147483647,NA,2147483647")
    assert value_fingerprint("logical:TRUE,FALSE,NA") == ("logical", "logical|3|T,F,NA")
    for invalid in (
        "raw:1",
        "integer:",
        "integer:01",
        "integer:+1",
        "integer:-0",
        "integer:2147483648",
        "integer:-2147483648",
        "integer:1,",
        "logical:T",
        "logical:FALSE,",
        "double:1.1",
        "double:NaN",
        "character:other",
        "character:builtin,special",
        "NULL:0",
    ):
        try:
            _ = value_fingerprint(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
    fixture, metadata = FIXTURE.read_text(), read_metadata()
    lines = fixture.splitlines()
    for index, column, replacement in (
        (1, 0, "length"),
        (1, 2, 'special:.Primitive("sum")'),
        (1, 2, 'builtin:.Primitive("length")'),
        (1, 2, "integer:1"),
        (1, 3, "FALSE"),
        (21, 0, ""),
        (21, 1, "ERROR"),
        (21, 2, ""),
        (21, 2, 'builtin:.Primitive("sum")'),
        (21, 3, "-"),
        (21, 3, "true"),
        (21, 4, "-"),
        (21, 4, "4A"),
        (21, 4, "0"),
        (21, 4, "ff"),
        (46, 2, ""),
        (46, 2, " padded "),
        (46, 3, "TRUE"),
    ):
        fields = lines[index - 1].split("\t")
        fields[column] = replacement
        try:
            _ = parse_row(index, "\t".join(fields), metadata)
        except (AssertionError, ValueError):
            continue
        raise AssertionError((index, column, replacement))
    for invalid in (
        "",
        fixture.rstrip("\n"),
        fixture + lines[0] + "\n",
        "\n".join(lines[:-1]) + "\n",
        fixture.replace("\n", "\r\n"),
        fixture.replace(lines[21], lines[20], 1),
        fixture.replace("\tTRUE\t", "\tFALSE\t", 1),
        fixture.replace("\t4142\n", "\t\n", 1),
    ):
        try:
            _ = parse_fixture(invalid, metadata)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid fixture accepted")
    for invalid in (lines[20] + "\textra", lines[20] + "\n", "\t".join(lines[20].split("\t")[:-1])):
        try:
            _ = parse_row(21, invalid, metadata)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid row accepted")


if __name__ == "__main__":
    self_check()
    print("Primitive value/profile reader and shared comparator self-checks passed; all 95 source rows supported")
