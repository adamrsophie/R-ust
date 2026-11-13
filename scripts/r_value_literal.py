# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/check-base-constants.py self-check
from __future__ import annotations

import json
import math
import re
import struct
from enum import StrEnum
from typing import assert_never


class Storage(StrEnum):
    LOGICAL = "logical"
    INTEGER = "integer"
    DOUBLE = "double"
    CHARACTER = "character"
    LIST = "list"


def real_literal(bits: str) -> str:
    assert re.fullmatch(r"[0-9a-f]{16}", bits), bits
    value = float(struct.unpack(">d", bytes.fromhex(bits))[0])
    if math.isnan(value):
        return "NA_real_" if int(bits, 16) & 0xFFFFFFFF == 1954 else "NaN"
    if math.isinf(value):
        return "Inf" if value > 0 else "-Inf"
    return repr(value) if value.is_integer() else value.hex()


def literal(fingerprint: str) -> str:
    """Decode only the independently recorded R fingerprint forms we represent."""
    if fingerprint == "NULL":
        return "NULL"
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
            values = [real_literal(value) for value in values]
            empty = "as.double(NULL)"
        case Storage.CHARACTER:
            assert all(value == "NA" or re.fullmatch(r"x(?:[0-9a-f]{2})*", value) for value in values)
            values = [
                "NA_character_" if value == "NA" else json.dumps(bytes.fromhex(value[1:]).decode()) for value in values
            ]
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


def self_check() -> None:
    for fingerprint, expected in (
        ("NULL", "NULL"),
        ("logical|0|", "as.logical(NULL)"),
        ("integer|0|", "as.integer(NULL)"),
        ("double|0|", "as.double(NULL)"),
        ("character|0|", "as.character(NULL)"),
        ("list|0|", "list()"),
        ("logical|3|T,F,NA", "c(TRUE,FALSE,NA)"),
        ("integer|2|-1,42", "c(-1L,42L)"),
        ("double|1|400921fb54442d18", "c(0x1.921fb54442d18p+1)"),
        ("double|1|7ff00000000007a2", "c(NA_real_)"),
        ("double|1|7ff80000000007a2", "c(NA_real_)"),
        ("double|1|fff00000000007a2", "c(NA_real_)"),
        ("double|1|7ff8000000000000", "c(NaN)"),
        ("double|2|7ff0000000000000,fff0000000000000", "c(Inf,-Inf)"),
        ("double|1|3ff00000000007a2", "c(0x1.00000000007a2p+0)"),
        ("double|1|8000000000000000", "c(-0.0)"),
        ("character|3|NA,x,x4e41", 'c(NA_character_,"","NA")'),
        ("list|2|NULL;NULL", "list(NULL,NULL)"),
    ):
        assert literal(fingerprint) == expected, fingerprint
    for invalid in (
        "raw|0|",
        "NULL|0|",
        "logical|1|X",
        "integer|1|NA",
        "double|1|nan",
        "double|1|7ff800000000000g",
        "character|1|x0",
        "character|1|bad",
        "list|1|1",
        "logical|2|T",
        "logical|-1|",
        "double|0|0000000000000000",
    ):
        try:
            _ = literal(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError(invalid)
