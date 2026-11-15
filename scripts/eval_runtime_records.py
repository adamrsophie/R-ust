"""Bounded typed records for the immutable 74-row eval reference."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Final

from dynamic_frame_records import Capture, JsonValue, decode, sequence, unhex

ROOT: Final = Path(__file__).resolve().parents[1]
FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/eval-runtime-r-4.6.0.tsv"
CASES: Final = ROOT / "scripts/eval-runtime-cases.tsv"
ERRORS: Final = ROOT / "scripts/eval-runtime-errors.tsv"
FIXTURE_SHA: Final = "d03e9b35b4abb5f7b3bf873c388177d47e7bcdaf27929f8b990b970c016b35ae"
CASES_SHA: Final = "b7e89c642a4be96c90d21d1c50d6ea2d0dc1e4b132c78da637f4cf9f335c379d"
ERRORS_SHA: Final = "5c7f667f8f9fb0d8dfd0484f50a0ab8f41d29ee6a9d6d58aaabf2775bbabaa08"
PRELUDE: Final = 'options(warn=1L, warnPartialMatchArgs=FALSE, keep.source=FALSE, digits=7L, scipen=0L, width=80L, deparse.cutoff=60L)'
SENTINEL: Final = b'cat("AFTER_SUBJECT_SENTINEL\\n")'
MODES: Final = ("expression", "file", "stdin", "rscript-file", "rscript-expression")
HEADER: Final = ("id", "category", "outcome", "node_hex", "typeof", "class_hex", "mode", "length",
                 "visible", "print_hex", "warnings_hex", "effects_hex", "status", "stdout_hex", "stderr_hex", "merged_hex")


@dataclass(frozen=True, slots=True)
class Warning:
    classes: tuple[str, ...]
    message: str
    call: str


@dataclass(frozen=True, slots=True)
class Typed:
    node: list[JsonValue]
    kind: str
    classes: tuple[str, ...]
    mode: str
    length: int
    visible: bool
    printed: bytes
    warnings: tuple[Warning, ...]
    effects: bytes


@dataclass(frozen=True, slots=True)
class Case:
    identifier: str
    category: str
    setup: bytes
    subject: bytes
    after: bytes
    status: int

    @property
    def source(self) -> bytes:
        return b"\n".join((PRELUDE.encode(), self.setup, self.subject, self.after, SENTINEL, b""))


@dataclass(frozen=True, slots=True)
class Row:
    case: Case
    typed: Typed | None
    error: str | None
    raw: Capture


def node_json(data: bytes) -> list[JsonValue]:
    value = decode(b'{"node":' + data + b'}')["node"]
    assert sequence(value), "node must be a JSON array"
    return value


def table(data: bytes, header: tuple[str, ...]) -> tuple[tuple[str, ...], ...]:
    assert data.endswith(b"\n") and b"\r" not in data, "invalid TSV framing"
    lines = data.decode("utf-8").splitlines()
    assert lines and lines[0] == "\t".join(header), "invalid TSV header"
    rows = tuple(tuple(line.split("\t")) for line in lines[1:])
    assert rows and all(len(row) == len(header) for row in rows), "invalid TSV width"
    assert len({row[0] for row in rows}) == len(rows), "duplicate identifier"
    return rows


def text_hex(value: str) -> str:
    return unhex(value).decode("utf-8")
