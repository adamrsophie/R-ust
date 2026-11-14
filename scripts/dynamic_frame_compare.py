"""Exact D1 raw comparison and bounded native diagnostic envelopes.

Condition messages were captured twice by dynamic_frame_conditions.R as a prelude;
only its exact added stderr markers were removed to prove original raw equality.
Frame policies encode EvalError Display and finish_parse_warnings, independently
of native captures. Frozen R diagnostics remain exact and are never line-stripped.
Run: python3 scripts/check-dynamic-frames.py self-check
"""
from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import Final

from dynamic_frame_records import Capture, Row, decode, normalized, sequence, sha, string, unhex

CONDITIONS: Final = Path(__file__).with_name("dynamic-frame-conditions.tsv")
OBSERVER_SHA: Final = "81d03ee2f7e32ac2f07f9f80cf0e3cb4ff71d0b99b2432d263db01234b76df2a"
CONDITIONS_SHA: Final = "2822e8b83a67de86f91f5f99d1e775cfce489e915b805c455fb041f9cd23a4f8"


class Frame(StrEnum):
    NONE = "none"
    DOMAIN = "domain"
    TYPE = "type"
    ARITY = "arity"
    STOP = "stop"


@dataclass(frozen=True, slots=True)
class Conditions:
    identifier: str
    source_sha256: str
    ordered: tuple[tuple[str, str], ...]
    frame: Frame


def read_conditions(rows: tuple[Row, ...], path: Path = CONDITIONS) -> dict[str, Conditions]:
    assert sha(CONDITIONS.with_name("dynamic_frame_conditions.R").read_bytes()) == OBSERVER_SHA
    data = path.read_bytes()
    assert sha(data) == CONDITIONS_SHA, "immutable condition metadata changed"
    lines = data.decode().splitlines()
    assert lines[0] == "id\tsource_sha256\tconditions_hex\tframe"
    result: dict[str, Conditions] = {}
    for line in lines[1:]:
        identifier, source, encoded, frame = line.split("\t")
        assert identifier not in result, "duplicate condition row"
        value = decode(b"{\"conditions\":" + unhex(encoded) + b"}")["conditions"]
        assert sequence(value)
        conditions: list[tuple[str, str]] = []
        for item in value:
            assert sequence(item) and len(item) == 2
            kind, payload = (string(part) for part in item)
            assert kind in ("warning", "error")
            conditions.append((kind, payload))
        errors = [payload for kind, payload in conditions if kind == "error"]
        assert len(errors) <= 1
        if errors:
            assert conditions[-1][0] == "error"
        policy = Frame(frame)
        assert bool(errors) == (policy != Frame.NONE)
        result[identifier] = Conditions(identifier, source, tuple(conditions), policy)
    assert set(result) == {row["id"] for row in rows}, "condition membership differs"
    for row in rows:
        condition = result[row["id"]]
        assert sha(row["source"]) == condition.source_sha256, "condition source differs"
        assert (condition.frame != Frame.NONE) == bool(row["raw"]["run-1"]["stdin"]["status"])
    return result


def warnings(messages: tuple[str, ...]) -> bytes:
    assert len(messages) <= 10, "unimplemented warning envelope"
    if not messages:
        return b""
    if len(messages) == 1:
        return ("Warning message:\n" + messages[0] + " \n").encode()
    return ("Warning messages:\n" + "".join(f"{i}: {text} \n" for i, text in enumerate(messages, 1))).encode()


def diagnostic(condition: Conditions) -> bytes:
    error = next((text for kind, text in condition.ordered if kind == "error"), None)
    messages = tuple(text for kind, text in condition.ordered if kind == "warning")
    if error is not None and error.startswith("(converted from warning) "):
        assert messages == (error.removeprefix("(converted from warning) "),)
        messages = ()
    tail = warnings(messages)
    if error is None:
        assert condition.frame == Frame.NONE
        return tail
    prefix = {Frame.DOMAIN: "domain error: ", Frame.TYPE: "type error: ",
              Frame.ARITY: "argument error: ", Frame.STOP: ""}[condition.frame]
    result = (prefix + error + "\n").encode()
    return result + (b"In addition: " + tail if tail else b"")


def raw_matches(gold: Capture, actual: Capture) -> bool:
    return gold["status"] == actual["status"] and all(
        normalized(unhex(gold[key])) == normalized(unhex(actual[key])) for key in ("stdout", "stderr", "merged")
    )


def expected_native(gold: Capture, condition: Conditions) -> Capture:
    stdout, stderr, merged = (unhex(gold[key]) for key in ("stdout", "stderr", "merged"))
    assert normalized(merged) == normalized(stdout + stderr), "unsupported interleaved diagnostic frame"
    assert bool(stderr) == bool(condition.ordered), "condition/raw presence differs"
    tail = diagnostic(condition)
    expected = gold.copy()
    expected["stderr"] = tail.hex()
    expected["merged"] = (stdout + tail).hex()
    return expected


def native_matches(gold: Capture, actual: Capture, condition: Conditions) -> bool:
    return raw_matches(expected_native(gold, condition), actual)
