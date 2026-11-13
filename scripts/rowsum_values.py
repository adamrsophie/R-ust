from __future__ import annotations

import math
import re
import struct
from dataclasses import dataclass
from enum import StrEnum
from typing import Final, assert_never

from scripts.constructor_reference import Case
from scripts.r_value_literal import literal

PROFILE: Final = "rowsum_canonical_storage_values_dims"


class Storage(StrEnum):
    INTEGER = "integer"
    DOUBLE = "double"
    LOGICAL = "logical"
    CHARACTER = "character"
    LIST = "list"


@dataclass(frozen=True, slots=True)
class Column:
    storage: Storage
    fingerprint: str


@dataclass(frozen=True, slots=True)
class Matrix:
    rows: int
    columns: int
    data: Column


@dataclass(frozen=True, slots=True)
class Frame:
    rows: int
    columns: tuple[Column, ...]


def size(text: str) -> int:
    assert re.fullmatch(r"0|[1-9][0-9]*", text)
    count = int(text)
    assert count <= 2147483647
    return count


def real_bits(value: str) -> str:
    if value in ("NA", "NaN"):
        return "7ff00000000007a2" if value == "NA" else "7ff8000000000000"
    if value in ("Inf", "-Inf"):
        return "7ff0000000000000" if value == "Inf" else "fff0000000000000"
    assert re.fullmatch(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?", value)
    number = float(value)
    assert math.isfinite(number)
    return struct.pack(">d", number).hex()


def parse_column(text: str, count: int) -> Column:
    storage_text, payload = text.split(":", 1)
    storage = Storage(storage_text)
    values = () if not payload else tuple(payload.split(","))
    assert len(values) == count
    match storage:
        case Storage.INTEGER:
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
        case Storage.DOUBLE:
            values = tuple(real_bits(value) for value in values)
        case Storage.LOGICAL:
            assert all(value in ("TRUE", "FALSE", "NA") for value in values)
            values = tuple({"TRUE": "T", "FALSE": "F", "NA": "NA"}[value] for value in values)
        case Storage.CHARACTER | Storage.LIST:
            assert not values
        case _:
            assert_never(storage)
    fingerprint = f"{storage.value}|{len(values)}|{','.join(values)}"
    _ = literal(fingerprint)
    return Column(storage, fingerprint)


def parse_value(text: str) -> Matrix | Frame:
    storage, shape, payload = text.split(":", 2)
    if storage == "df":
        rows = size(shape)
        parts = () if not payload else tuple(payload.split(";"))
        columns = tuple(parse_column(part, rows) for part in parts)
        assert all(column.storage in (Storage.INTEGER, Storage.DOUBLE) for column in columns)
        return Frame(rows, columns)
    rows_text, columns_text = shape.split(",")
    rows, columns = size(rows_text), size(columns_text)
    return Matrix(rows, columns, parse_column(storage + ":" + payload, rows * columns))


def column_checks(column: Column, path: str) -> tuple[str, ...]:
    if column.storage == Storage.LIST:
        return (f'identical(typeof({path}),"list")', f"identical(length({path}),0L)")
    return (
        f'identical(typeof({path}),"{column.storage.value}")',
        f"identical(as.{column.storage.value}({path}),{literal(column.fingerprint)})",
    )


def checks(expected: Matrix | Frame, names: str) -> tuple[str, ...]:
    if isinstance(expected, Matrix):
        assert not names
        return (
            f"identical(dim(answer),c({expected.rows}L,{expected.columns}L))",
            *column_checks(expected.data, "answer"),
        )
    assert names
    result = [
        'identical(typeof(answer),"list")',
        'identical(class(answer),"data.frame")',
        f"identical(dim(answer),c({expected.rows}L,{len(expected.columns)}L))",
        f"identical(names(answer),{literal(names)})",
    ]
    for index, column in enumerate(expected.columns, 1):
        result.extend(column_checks(column, f"answer[[{index}L]]"))
    return tuple(result)


def program(row: Case, names: str = "") -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    return (
        "options(warn=1);answer<-{"
        + row.source
        + "};cat("
        + "&&".join(checks(parse_value(row.fingerprint), names))
        + ',"\\n",sep="")\n'
    )
