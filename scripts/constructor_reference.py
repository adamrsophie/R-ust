from __future__ import annotations

import json
import re
import struct
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

from scripts.stop_capture import Run, digest, stream_matches

ROOT: Final = Path(__file__).resolve().parent.parent
FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/constructors-r-4.6.0.tsv"
FIXTURE_SHA: Final = "fd596a10ff3af27a3b4c77430e81da53d15a902faf921e4e59e7d5aa5d56c57d"
SELECTED: Final = (
    1,
    4,
    6,
    8,
    9,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    19,
    20,
    22,
    23,
    25,
    26,
    27,
    28,
    29,
    30,
    32,
    33,
    34,
    35,
    39,
    40,
    42,
    45,
    46,
    47,
    48,
    49,
    51,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    63,
    114,
    119,
    122,
    178,
    181,
    237,
    240,
    522,
    523,
    527,
    533,
    538,
    558,
    559,
    586,
    588,
    589,
    592,
    595,
    598,
    601,
    604,
    607,
    610,
    612,
    613,
    615,
    616,
    618,
    619,
    620,
    622,
    624,
    *range(625, 649),
)


class Lane(StrEnum):
    REPRESENTED = "represented"
    FUTURE = "future-type"


class Storage(StrEnum):
    LOGICAL = "logical"
    INTEGER = "integer"
    DOUBLE = "double"
    CHARACTER = "character"
    LIST = "list"


@dataclass(frozen=True, slots=True)
class Case:
    identifier: int
    lane: Lane
    source: str
    storage: str
    fingerprint: str
    error: str
    warnings: tuple[str, ...]
    stdout: str


def decode_hex(text: str) -> str:
    if text == "-":
        return ""
    assert re.fullmatch(r"(?:[0-9a-f]{2})+", text), text
    return bytes.fromhex(text).decode()


def literal(fingerprint: str) -> str:
    kind, count_text, payload = fingerprint.split("|", 2)
    storage = Storage(kind)
    count = int(count_text)
    assert count >= 0
    values = payload.split(",") if payload else []
    match storage:
        case Storage.LOGICAL:
            assert all(value in ("T", "F", "NA") for value in values)
            values = [{"T": "TRUE", "F": "FALSE", "NA": "NA"}[value] for value in values]
            empty = "as.logical(NULL)"
        case Storage.INTEGER:
            values = [str(int(value)) + "L" for value in values]
            empty = "as.integer(NULL)"
        case Storage.DOUBLE:
            assert all(re.fullmatch(r"[0-9a-f]{16}", value) for value in values)
            values = [repr(struct.unpack(">d", bytes.fromhex(value))[0]) for value in values]
            empty = "as.double(NULL)"
        case Storage.CHARACTER:
            assert all(value.startswith("x") for value in values)
            values = [json.dumps(bytes.fromhex(value[1:]).decode()) for value in values]
            empty = "as.character(NULL)"
        case Storage.LIST:
            values = payload.split(";") if payload else []
            assert all(value == "NULL" for value in values)
            assert len(values) == count
            return "list(" + ",".join(values) + ")"
        case _:
            assert_never(storage)
    assert len(values) == count
    return "c(" + ",".join(values) + ")" if values else empty


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"
    rows: list[Case] = []
    for expected_id, line in enumerate(lines[1:], 1):
        identifier, lane, source, storage, fingerprint, error, warnings, stdout = line.split("\t")
        assert int(identifier) == expected_id
        row = Case(
            expected_id,
            Lane(lane),
            source,
            storage,
            fingerprint,
            decode_hex(error),
            tuple(decode_hex(warnings).split("\n")) if warnings != "-" else (),
            decode_hex(stdout),
        )
        assert bool(row.error) == (storage == "error")
        if storage == "error":
            assert fingerprint == "-"
        match row.lane:
            case Lane.REPRESENTED:
                if storage != "error":
                    assert fingerprint.split("|", 1)[0] == storage
                    _ = literal(fingerprint)
            case Lane.FUTURE:
                assert storage in ("error", "NULL", "complex", "raw", "pairlist", "expression")
            case _:
                assert_never(row.lane)
        rows.append(row)
    assert len(rows) == 648 and sum(row.lane == Lane.REPRESENTED for row in rows) == 628
    assert len(SELECTED) == len(set(SELECTED)) == 102
    assert all(rows[identifier - 1].lane == Lane.REPRESENTED for identifier in SELECTED)
    assert all(not row.stdout for row in rows if row.warnings), "ordered effects plus warnings need explicit events"
    return tuple(rows)


def program(row: Case) -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    return (
        "options(warn=1);answer<-{"
        + row.source
        + "};cat(identical(answer,"
        + literal(row.fingerprint)
        + '),"\\n",sep="")\n'
    )


def stream_valid(text: str, row: Case, *, merged: bool, reference: bool) -> bool:
    tail = text
    for warning in row.warnings:
        header = r"(?:Warning in [^\n]*? :[ \t]*(?:\n  )?|Warning: )" if reference else r"Warning: "
        match = re.match(header + re.escape(warning + "\n"), tail)
        if match is None:
            return False
        tail = tail[match.end() :]
    output = row.stdout + ("" if row.error else "TRUE\n") if merged else ""
    return stream_matches(tail, output, row.error, error=bool(row.error), reference=reference)


def matches(row: Case, runs: tuple[Run, Run], *, reference: bool) -> bool:
    separate, merged = runs
    output = row.stdout + ("" if row.error else "TRUE\n")
    return (
        separate.status == merged.status == int(bool(row.error))
        and separate.stdout == output
        and not merged.stderr
        and stream_valid(separate.stderr, row, merged=False, reference=reference)
        and stream_valid(merged.stdout, row, merged=True, reference=reference)
    )


def self_check() -> None:
    row = Case(
        1,
        Lane.REPRESENTED,
        "",
        "error",
        "-",
        "first\nCalls: literal\nExecution halted\nlast",
        ("NAs introduced by coercion", "partial argument match of 'len' to 'length'"),
        "AB\n",
    )
    for reference, warning_header in ((False, "Warning: "), (True, "Warning in f() : "), (True, "Warning: ")):
        warnings = "".join(warning_header + text + "\n" for text in row.warnings)
        raw = warnings + "AB\nError: " + row.error + "\n" + ("Execution halted\n" if reference else "")
        assert stream_valid(raw, row, merged=True, reference=reference)
        for changed in (
            raw[1:],
            raw + "extra\n",
            raw.replace("AB", "BA"),
            raw.replace("first", "other"),
            raw.replace(warnings, ""),
            raw.replace(warnings, warnings + warnings),
            raw.replace(warnings, warnings.splitlines(keepends=True)[1] + warnings.splitlines(keepends=True)[0]),
            raw.replace("Calls: literal\nExecution halted\n", ""),
        ):
            assert not stream_valid(changed, row, merged=True, reference=reference), repr(changed)
    assert literal("logical|0|") == "as.logical(NULL)"
    assert literal("double|1|4045000000000000") == "c(42.0)"
    assert literal("character|2|x,x6162") == 'c("","ab")'
    assert literal("list|2|NULL;NULL") == "list(NULL,NULL)"
