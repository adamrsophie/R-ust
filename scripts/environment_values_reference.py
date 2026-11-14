#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run:
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# python3 scripts/check-environment-values.py self-check
from __future__ import annotations

import csv
import hashlib
import os
import re
import subprocess
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

from scripts.stop_capture import Invocation, Run, digest, encode_bytes, stream_matches

ROOT: Final = Path(__file__).resolve().parent.parent
FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/environment-values-r-4.6.0.tsv"
FIXTURE_SHA: Final = "3cda1fa2849f25dce776684092d62d4d85700e335f90546fdc89db2c164414d3"
GOLD_SHA: Final = "73dea21f4350c7d899d9834b5b726bff43974a5411edbe4dd80781768565fad1"
SCOPE_SHA: Final = "6c5c0f3cf9e0a45163d075ae62add69e22cfcc53494d73afb658eef4608a9bab"
PAYLOAD_SHA: Final = "80340856f0a574b0617c3536f7208a751206cdd7bb5915162dedc0cbe42a6af9"
ENV: Final = dict(os.environ, LC_ALL="C", LANG="C", R_ENABLE_JIT="0", R_DEFAULT_PACKAGES="base")
POINTER: Final = re.compile(r"(?<=<environment: )0x[0-9a-fA-F]+(?=>)")
PROFILE: Final = "4.6.0|aarch64-apple-darwin23|8|53|C|0|.GlobalEnv,Autoloads,package:base"
HEADER: Final = (
    "id", "category", "outcome", "node_hex", "typeof", "class_hex", "mode", "length",
    "visible", "print_hex", "warnings_hex", "effects_hex", "status", "stdout_hex",
    "stderr_hex", "merged_hex", "surface", "source_hex", "payload_hex", "payload_stderr_hex",
)


class Surface(StrEnum):
    CLI = "cli"
    SDK = "sdk"
    REFERENCE = "reference_only"


@dataclass(frozen=True, slots=True)
class Streams:
    stdout: str
    stderr: str
    merged: str


@dataclass(frozen=True, slots=True)
class Reference:
    identifier: str
    surface: Surface
    source: bytes
    status: int
    streams: Streams
    error: str | None
    warning: str


def parse_row(fields: list[str]) -> Reference:
    assert len(fields) == len(HEADER), "invalid reference column count"
    identifier = fields[0]
    assert re.fullmatch(r"[a-z][a-z0-9_]*", identifier), "invalid reference identifier"
    status = int(fields[12])
    assert status in (0, 1), "invalid reference status"
    surface = Surface(fields[16])
    source = bytes.fromhex(fields[17])
    assert source and source.isascii(), "reference source must be nonempty ASCII bytes"
    output = Streams(*(bytes.fromhex(fields[i]).decode() for i in (13, 14, 15)))
    warnings = bytes.fromhex(fields[10]).decode().splitlines()
    assert len(warnings) % 3 == 0, "invalid warning metadata"
    warning = "".join("Warning: " + warnings[i] + "\n" for i in range(1, len(warnings), 3))
    error = bytes.fromhex(fields[18]).decode() if status else None
    assert bytes.fromhex(fields[19]).decode() == (output.stderr if status else "")
    assert not (error is not None and warning)
    return Reference(identifier, surface, source, status, output, error, warning)


def read_reference() -> tuple[Reference, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA, "immutable environment fixture changed"
    with FIXTURE.open(newline="") as stream:
        rows = list(csv.reader((line for line in stream if not line.startswith("#")), delimiter="\t"))
    assert tuple(rows[0]) == HEADER
    parsed = tuple(parse_row(row) for row in rows[1:])
    assert len(parsed) == len({row.identifier for row in parsed}) == 128
    assert tuple(sum(row.surface == scope for row in parsed) for scope in Surface) == (97, 28, 3)
    original = "\n".join("\t".join(row[:16]) for row in rows) + "\n"
    scope = "id\tsurface\n" + "".join(f"{row[0]}\t{row[16]}\n" for row in rows[1:])
    payload = "id\tpayload_hex\tstderr_hex\n" + "".join(
        f"{row[0]}\t{row[18]}\t{row[19]}\n" for row in rows[1:] if row[12] == "1"
    )
    for content, expected in ((original, GOLD_SHA), (scope, SCOPE_SHA), (payload, PAYLOAD_SHA)):
        assert hashlib.sha256(content.encode()).hexdigest() == expected
    return parsed


def normalized(text: str) -> str:
    return POINTER.sub("0xPOINTER", text)


def cli_rows(rows: tuple[Reference, ...]) -> tuple[Reference, ...]:
    selected: list[Reference] = []
    for row in rows:
        match row.surface:
            case Surface.CLI:
                selected.append(row)
            case Surface.SDK | Surface.REFERENCE:
                continue
            case _:
                assert_never(row.surface)
    assert len(selected) == 97
    return tuple(selected)


def capture(request: Invocation, merged: bool) -> Run:
    result = subprocess.run(
        request.command, input=request.stdin, env=ENV, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT if merged else subprocess.PIPE, timeout=30, check=False,
    )
    stdout, stderr = result.stdout, result.stderr or b""
    return Run(request, result.returncode, stdout.decode(), stderr.decode(), encode_bytes(stdout), encode_bytes(stderr))


def capture_pair(request: Invocation) -> tuple[Run, Run]:
    return capture(request, False), capture(request, True)


def live_profile(executables: tuple[str, str]) -> tuple[str, str]:
    source = (
        'cat(paste(getRversion(),R.version$platform,.Machine$sizeof.longdouble,'
        '.Machine$double.digits,Sys.getlocale("LC_CTYPE"),compiler::enableJIT(-1L),'
        'paste(search(),collapse=","),sep="|"))'
    )
    values: list[str] = []
    for executable in executables:
        run = capture(Invocation((executable, "--vanilla", "--slave", "-e", source), b""), False)
        assert run.status == 0 and run.stderr == "" and run.stdout == PROFILE, run
        values.append(run.stdout)
    return values[0], values[1]


def reference_matches(row: Reference, separate: Run, merged: Run) -> bool:
    return (
        separate.status == merged.status == row.status
        and normalized(separate.stdout) == normalized(row.streams.stdout)
        and normalized(separate.stderr) == normalized(row.streams.stderr)
        and normalized(merged.stdout) == normalized(row.streams.merged)
        and merged.stderr == ""
    )


def native_matches(row: Reference, reference: tuple[Run, Run], native: tuple[Run, Run]) -> bool:
    gold, gold_merged = reference
    separate, merged = native
    assert reference_matches(row, gold, gold_merged), "fresh R differs from immutable fixture"
    common = (
        separate.status == merged.status == row.status
        and normalized(separate.stdout) == normalized(gold.stdout)
        and merged.stderr == ""
    )
    if row.error is not None:
        assert stream_matches(gold.stderr, "", row.error, error=True, reference=True), "R payload differs"
        return (
            common
            and stream_matches(separate.stderr, "", row.error, error=True, reference=False)
            and stream_matches(normalized(merged.stdout), normalized(gold.stdout), row.error, error=True, reference=False)
        )
    wanted = normalized(gold_merged.stdout)
    if gold.stderr:
        assert wanted.count(normalized(gold.stderr)) == 1
        wanted = wanted.replace(normalized(gold.stderr), row.warning, 1)
    else:
        assert row.warning == ""
    return common and separate.stderr == row.warning and normalized(merged.stdout) == wanted
