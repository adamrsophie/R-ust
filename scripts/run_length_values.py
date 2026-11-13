from __future__ import annotations

import math
import re
import struct
from dataclasses import dataclass
from enum import StrEnum
from typing import Final, assert_never

from scripts.constructor_reference import Case
from scripts.r_value_literal import literal

PROFILE: Final = "canonical_storage_values"


class Kind(StrEnum):
    NULL = "null"
    INTEGER = "integer"
    LOGICAL = "logical"
    DOUBLE = "double"
    CHARACTER = "character"
    LIST = "list"
    FACTOR = "factor"


@dataclass(frozen=True, slots=True)
class Value:
    kind: Kind
    fingerprint: str
    names: tuple[str, ...] = ()
    children: tuple[Value, ...] = ()


def character_values(payload: str) -> tuple[str, ...]:
    if not payload:
        return ()
    tokens = tuple(match.group() for match in re.finditer(r'NA|"(?:[^"\\\x00-\x1f]|\\.)*"', payload))
    assert ",".join(tokens) == payload
    values: list[str] = []
    for token in tokens:
        if token == "NA":
            values.append("NA")
        else:
            decoded = character_token(token)
            values.append("x" + decoded.encode().hex())
    return tuple(values)


def character_token(token: str) -> str:
    pieces: list[str] = []
    index = 1
    escapes = {'"': '"', "\\": "\\", "/": "/", "b": "\b", "f": "\f", "n": "\n", "r": "\r", "t": "\t"}
    while index < len(token) - 1:
        char = token[index]
        index += 1
        if char != "\\":
            pieces.append(char)
            continue
        escape = token[index]
        index += 1
        if escape in escapes:
            pieces.append(escapes[escape])
            continue
        assert escape == "u"
        digits = token[index : index + 4]
        assert re.fullmatch(r"[0-9a-fA-F]{4}", digits)
        code = int(digits, 16)
        index += 4
        if 0xD800 <= code <= 0xDBFF:
            assert token[index : index + 2] == "\\u"
            low_digits = token[index + 2 : index + 6]
            assert re.fullmatch(r"[0-9a-fA-F]{4}", low_digits)
            low = int(low_digits, 16)
            assert 0xDC00 <= low <= 0xDFFF
            code = 0x10000 + ((code - 0xD800) << 10) + low - 0xDC00
            index += 6
        assert not 0xD800 <= code <= 0xDFFF
        pieces.append(chr(code))
    assert index == len(token) - 1
    result = "".join(pieces)
    assert "\0" not in result
    return result


def atomic_values(kind: Kind, payload: str) -> tuple[str, ...]:
    if kind == Kind.CHARACTER:
        return character_values(payload)
    values = () if not payload else tuple(payload.split(","))
    match kind:
        case Kind.INTEGER:
            assert all(re.fullmatch(r"(?:NA|0|-?[1-9][0-9]*)", value) for value in values)
            assert all(value == "NA" or -2147483647 <= int(value) <= 2147483647 for value in values)
        case Kind.LOGICAL:
            assert all(value in ("TRUE", "FALSE", "NA") for value in values)
            values = tuple({"TRUE": "T", "FALSE": "F", "NA": "NA"}[value] for value in values)
        case Kind.DOUBLE:
            encoded: list[str] = []
            for value in values:
                if value in ("NA", "NaN"):
                    encoded.append("7ff00000000007a2" if value == "NA" else "7ff8000000000000")
                else:
                    assert re.fullmatch(r"[0-9a-f]{16}", value)
                    bits = bytes.fromhex(value)
                    assert not math.isnan(struct.unpack("<d", bits)[0])
                    encoded.append(bits[::-1].hex())
            values = tuple(encoded)
        case Kind.NULL | Kind.LIST | Kind.FACTOR:
            raise AssertionError(kind)
        case _:
            assert_never(kind)
    return values


def parse_value(text: str) -> Value:
    if text == "null":
        return Value(Kind.NULL, "NULL")
    kind_text, payload = text.split(":", 1)
    kind = Kind(kind_text)
    match kind:
        case Kind.LIST:
            if payload.startswith("lengths="):
                lengths, values = payload.removeprefix("lengths=").split(";values=", 1)
                children = (parse_value(lengths), parse_value(values))
                assert children[0].kind == Kind.INTEGER
                return Value(kind, "", ("lengths", "values"), children)
            entries = () if not payload else tuple(payload.split(";"))
            pairs = tuple(entry.split("=", 1) for entry in entries)
            assert all(len(pair) == 2 for pair in pairs)
            names = tuple(pair[0] for pair in pairs)
            assert all(not name or re.fullmatch(r"[A-Za-z.][A-Za-z0-9._]*", name) for name in names)
            children = tuple(parse_value(pair[1]) for pair in pairs)
            assert all(child.kind != Kind.LIST for child in children)
            return Value(kind, "", names, children)
        case Kind.FACTOR:
            codes, levels = payload.split(":character:", 1)
            children = (parse_value(codes), parse_value("character:" + levels))
            assert children[0].kind == Kind.INTEGER
            level_count = int(children[1].fingerprint.split("|", 2)[1])
            code_values = children[0].fingerprint.split("|", 2)[2]
            assert all(value == "NA" or 1 <= int(value) <= level_count for value in code_values.split(",") if value)
            return Value(kind, "", (), children)
        case Kind.INTEGER | Kind.LOGICAL | Kind.DOUBLE | Kind.CHARACTER:
            values = atomic_values(kind, payload)
            fingerprint = f"{kind.value}|{len(values)}|{','.join(values)}"
            _ = literal(fingerprint)
            return Value(kind, fingerprint)
        case Kind.NULL:
            raise AssertionError(text)
        case _:
            assert_never(kind)


def character_literal(values: tuple[str, ...]) -> str:
    return literal(f"character|{len(values)}|" + ",".join("x" + value.encode().hex() for value in values))


def checks(value: Value, path: str) -> tuple[str, ...]:
    match value.kind:
        case Kind.LIST:
            names = character_literal(value.names) if any(value.names) else "NULL"
            result = [
                f'identical(typeof({path}),"list")',
                f"identical(length({path}),{len(value.children)}L)",
                f"identical(names({path}),{names})",
            ]
            for index, child in enumerate(value.children, 1):
                result.extend(checks(child, f"{path}[[{index}L]]"))
            return tuple(result)
        case Kind.FACTOR:
            return (
                f'identical(typeof({path}),"integer")',
                f'identical(class({path}),"factor")',
                *checks(value.children[0], f"as.integer({path})"),
                *checks(value.children[1], f"levels({path})"),
            )
        case Kind.DOUBLE:
            expected = literal(value.fingerprint)
            actual = f"as.double({path})"
            return (
                f'identical(typeof({path}),"double")',
                f"identical({actual},{expected})",
                f"identical(1/({actual}),1/({expected}))",
            )
        case Kind.INTEGER | Kind.LOGICAL | Kind.CHARACTER:
            return (
                f'identical(typeof({path}),"{value.kind.value}")',
                f"identical(as.{value.kind.value}({path}),{literal(value.fingerprint)})",
            )
        case Kind.NULL:
            return (f"identical({path},{literal(value.fingerprint)})",)
        case _:
            assert_never(value.kind)


def program(row: Case) -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    expected = parse_value(row.fingerprint)
    return (
        "options(warn=1);answer<-{" + row.source + "};cat(" + "&&".join(checks(expected, "answer")) + ',"\\n",sep="")\n'
    )
