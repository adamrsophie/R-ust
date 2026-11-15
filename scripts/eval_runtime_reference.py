"""Strict 74-row source/typed/raw binding; every row is CLI and SDK scope."""
from __future__ import annotations

import re

from dynamic_frame_records import Capture, Invocation, sha, unhex
from eval_runtime_nodes import metadata, node
from eval_runtime_records import CASES, CASES_SHA, ERRORS, ERRORS_SHA, FIXTURE, FIXTURE_SHA, HEADER
from eval_runtime_records import Case, Row, Typed, Warning, node_json, table, text_hex

CASE_HEADER = ("id", "category", "setup_hex", "subject_hex", "after_hex", "expected_status")
ERROR_HEADER = ("id", "payload_hex", "stderr_hex")


def cases(data: bytes) -> tuple[Case, ...]:
    result: list[Case] = []
    for identifier, category, setup, subject, after, status in table(data, CASE_HEADER):
        assert re.fullmatch(r"[a-z][a-z0-9_]*", identifier)
        assert re.fullmatch(r"[a-z][a-z0-9_]*", category)
        assert status in ("0", "1")
        values = tuple(unhex(value) for value in (setup, subject, after))
        assert values[1] and all(value.isascii() and b"\x00" not in value for value in values)
        result.append(Case(identifier, category, values[0], values[1], values[2], int(status)))
    assert len(result) == 74, "complete 74-row source inventory required"
    return tuple(result)


def warning(value: str) -> tuple[Warning, ...]:
    if not value:
        return ()
    classes, message, call = text_hex(value).split("\n", 2)
    tags = tuple(classes.split(","))
    assert tags in (("simpleWarning", "warning", "condition"), ("partialMatchWarning", "warning", "condition"))
    assert message and call
    return (Warning(tags, message, call),)


def typed(fields: tuple[str, ...]) -> Typed:
    value = node_json(unhex(fields[3]))
    kind, length = node(value)
    assert fields[4] == kind and fields[7] == str(length), "node/type/length disagree"
    classes = tuple(text_hex(fields[5]).split(","))
    expected_class, expected_mode = metadata(kind)
    assert classes == (expected_class,) and fields[6] == expected_mode, "unmeasured attributes/mode"
    assert fields[8] in ("TRUE", "FALSE")
    return Typed(value, kind, classes, fields[6], length, fields[8] == "TRUE", unhex(fields[9]),
                 warning(fields[10]), unhex(fields[11]))


def parse(data: bytes, inputs: tuple[Case, ...], errors: bytes) -> tuple[Row, ...]:
    rows = table(data, HEADER)
    payloads = {identifier: (text_hex(payload), unhex(stderr))
                for identifier, payload, stderr in table(errors, ERROR_HEADER)}
    assert len(rows) == len(inputs) == 74 and len(payloads) == 22
    result: list[Row] = []
    for fields, case in zip(rows, inputs, strict=True):
        assert fields[:2] == (case.identifier, case.category), "source membership/order differs"
        assert fields[12] == str(case.status) and fields[2] in ("value", "error")
        for index, name in enumerate(HEADER):
            if name.endswith("_hex"):
                _ = unhex(fields[index])
        streams = tuple(unhex(fields[index]) for index in (13, 14, 15))
        for stream in streams:
            _ = stream.decode("utf-8")
        value = None
        error = None
        if fields[2] == "error":
            assert case.status == 1 and all(part == "" for part in fields[3:12])
            error, stderr = payloads[case.identifier]
            assert error and stderr == streams[1] and b"AFTER_SUBJECT_SENTINEL" not in streams[0]
        else:
            assert case.status == 0 and case.identifier not in payloads
            value = typed(fields)
            assert streams[0].endswith(b"AFTER_SUBJECT_SENTINEL\n")
            assert bool(streams[1]) == bool(value.warnings), "spurious/missing warning"
        raw = Capture(stdout=fields[13], stderr=fields[14], merged=fields[15], status=case.status,
                      invocation=Invocation(argv=["R", "--vanilla", "--slave", "-f", case.identifier + ".R"], stdin=None))
        result.append(Row(case, value, error, raw))
    assert sum(row.typed is not None for row in result) == 52
    assert {row.case.identifier for row in result if row.error is not None} == set(payloads)
    return tuple(result)


def read_data(data: bytes, inputs: bytes, errors: bytes) -> tuple[Row, ...]:
    assert sha(data) == FIXTURE_SHA, "immutable typed/raw fixture changed"
    assert sha(inputs) == CASES_SHA, "immutable source cases changed"
    assert sha(errors) == ERRORS_SHA, "immutable error payloads changed"
    rows = parse(data, cases(inputs), errors)
    base = next(row for row in rows if row.case.identifier == "eval_base_environment")
    assert base.typed is not None and base.typed.node == ["logical", "FALSE"]
    return rows


def read_reference() -> tuple[Row, ...]:
    return read_data(FIXTURE.read_bytes(), CASES.read_bytes(), ERRORS.read_bytes())
