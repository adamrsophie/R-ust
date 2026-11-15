"""Validate the frozen JSON node transport, never parse or execute R source."""
from __future__ import annotations

import re
from typing import Final

from dynamic_frame_records import JsonValue, sequence, string, unhex

ATOMIC: Final = {"integer": "integer", "double": "real", "logical": "logical", "character": "string"}


def children(value: JsonValue) -> list[JsonValue]:
    assert sequence(value), "node children must be an array"
    return value


def arguments(value: JsonValue, depth: int) -> int:
    rows = children(value)
    for row in rows:
        parts = children(row)
        assert len(parts) == 2
        name, child = parts
        if name is not None:
            assert isinstance(name, str) and unhex(name), "argument tags cannot be empty"
            _ = unhex(name).decode("utf-8")
        _ = node(child, depth + 1)
    return len(rows)


def node(value: JsonValue, depth: int = 0) -> tuple[str, int]:
    assert depth < 64, "node nesting limit exceeded"
    fields = children(value)
    assert fields, "empty node"
    tag = string(fields[0])
    if tag in ("nil", "missing"):
        assert len(fields) == 1
        return ("NULL", 0) if tag == "nil" else ("symbol", 1)
    if tag in ("symbol", "primitive", "real", "integer", "logical"):
        assert len(fields) == 2
        payload = string(fields[1])
        if tag == "symbol":
            assert unhex(payload) and "\x00" not in unhex(payload).decode("utf-8")
        elif tag == "primitive":
            assert payload == "quote"
        elif tag == "real":
            assert len(unhex(payload)) == 8
        elif tag == "integer":
            assert payload == "NA" or re.fullmatch(r"-?(?:0|[1-9][0-9]*)", payload)
            assert payload == "NA" or -2147483647 <= int(payload) <= 2147483647
        else:
            assert payload in ("TRUE", "FALSE", "NA")
        return {"symbol": "symbol", "primitive": "special", "real": "double"}.get(tag, tag), 1
    if tag == "string":
        assert len(fields) == 3
        payload = fields[1]
        assert payload is None or isinstance(payload, str)
        if payload is not None:
            assert b"\x00" not in unhex(payload)
            _ = unhex(payload).decode("utf-8")
        assert fields[2] in ("unknown", "UTF-8", "latin1", "bytes")
        return "character", 1
    if tag == "call":
        assert len(fields) == 3
        _ = node(fields[1], depth + 1)
        return "language", arguments(fields[2], depth) + 1
    if tag == "pairlist":
        assert len(fields) == 2
        length = arguments(fields[1], depth)
        assert length > 0, "empty pairlists serialize as nil"
        return "pairlist", length
    assert tag == "vector" and len(fields) == 3, "invalid node tag/arity"
    kind = string(fields[1])
    assert kind in (*ATOMIC, "list", "expression")
    values = children(fields[2])
    for child in values:
        child_kind, length = node(child, depth + 1)
        if kind in ATOMIC:
            assert child_kind == kind and length == 1, "heterogeneous atomic vector"
            assert children(child)[0] == ATOMIC[kind], "nested atomic vector"
    return kind, len(values)


def metadata(kind: str) -> tuple[str, str]:
    return ({"symbol": "name", "language": "call", "special": "function", "double": "numeric"}.get(kind, kind),
            {"symbol": "name", "language": "call", "special": "function", "integer": "numeric", "double": "numeric"}.get(kind, kind))
