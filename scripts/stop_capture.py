from __future__ import annotations

import base64
import hashlib
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal, assert_never

from scripts.stop_reference import Reference, environment

Mode = Literal["expression", "file", "stdin", "rscript-file", "rscript-expression"]
MODES: Final[tuple[Mode, ...]] = ("expression", "file", "stdin", "rscript-file", "rscript-expression")
FIXTURE_SHA: Final = "7c03482d9d849ac9ae5b2e44b8a432ffdbca4f7533ca7ab9dfde317fd1e3ac84"


@dataclass(frozen=True, slots=True)
class Invocation:
    command: tuple[str, ...]
    stdin: bytes


@dataclass(frozen=True, slots=True)
class Run:
    invocation: Invocation
    status: int
    stdout: str
    stderr: str
    stdout_base64: str
    stderr_base64: str


@dataclass(frozen=True, slots=True)
class Comparison:
    mode: Mode
    reference: Reference
    r: Run
    native: Run
    r_merged: Run
    native_merged: Run
    supported: bool
    passed: bool
    exact_streams: bool
    exact_merged: bool


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def encode_bytes(value: bytes) -> str:
    return base64.b64encode(value).decode()


def invocation(mode: Mode, executables: tuple[str, str], source: Path) -> Invocation:
    match mode:
        case "stdin":
            return Invocation((executables[0], "--vanilla", "--no-echo"), source.read_bytes())
        case "file":
            return Invocation((executables[0], "--vanilla", "--no-echo", "-f", str(source)), b"")
        case "rscript-file":
            return Invocation((executables[1], "--vanilla", str(source)), b"")
        case "expression" | "rscript-expression":
            prefix = (
                (executables[0], "--vanilla", "--no-echo") if mode == "expression" else (executables[1], "--vanilla")
            )
            return Invocation((*prefix, "-e", source.read_text().replace("\\", "\\\\")), b"")
        case _:
            assert_never(mode)


def run(request: Invocation, *, merged: bool = False) -> Run:
    result = subprocess.run(
        request.command,
        input=request.stdin,
        env=environment(),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT if merged else subprocess.PIPE,
        timeout=30,
        check=False,
    )
    stdout, stderr = result.stdout, result.stderr or b""
    return Run(request, result.returncode, stdout.decode(), stderr.decode(), encode_bytes(stdout), encode_bytes(stderr))


def stream_matches(text: str, output: str, message: str, *, error: bool, reference: bool) -> bool:
    """Match exact ordered effects and message inside the permitted CLI envelope."""
    if not text.startswith(output):
        return False
    tail = text[len(output) :]
    if not error:
        return tail == ""
    if not reference and tail == message + "\n":
        return True
    header = r"(?:Error in [^\n]*? : (?:\n  )?|Error: )"
    if not reference:
        header = r"(?:Error in [^\n]*? : (?:\n  )?|Error: |(?:argument|domain|control-flow|type) error: )"
    footer = r"(?:Calls: [^\n]*\n)?Execution halted\n" if reference else ""
    framing = re.match(header, tail)
    return framing is not None and re.fullmatch(re.escape(message + "\n") + footer, tail[framing.end() :]) is not None


def matches(row: Reference, separate: Run, merged: Run, *, reference: bool) -> bool:
    error = row.kind == "error"
    output = row.stdout if error else row.stdout + f"{row.kind}:{row.value}\n"
    return (
        separate.status == merged.status == int(error)
        and separate.stdout == output
        and merged.stderr == ""
        and stream_matches(separate.stderr, "", row.message, error=error, reference=reference)
        and stream_matches(merged.stdout, output, row.message, error=error, reference=reference)
    )


def comparator_self_check() -> None:
    for message in (
        "",
        "boom",
        "first\nsecond",
        "Error: literal",
        "Calls: literal",
        "Execution halted",
        "Error: literal\nCalls: literal\nExecution halted",
    ):
        text = "AB" + message + "\n"
        assert stream_matches(text, "AB", message, error=True, reference=False)
        assert not stream_matches(text, "AB", message, error=True, reference=True)
        for changed in (
            text[1:],
            "X" + text,
            text + "extra\n",
            "BA" + message + "\n",
            "ABchanged\n",
            "AB" + (message[1:] if message else "x") + "\n",
            "AB" + (message[::-1] if len(message) > 1 else "x") + "\n",
        ):
            assert not stream_matches(changed, "AB", message, error=True, reference=False), repr(changed)
        assert not stream_matches("AB\n", "AB", message + "extra", error=True, reference=False)
    for message in ("", "first\nsecond", "first\nCalls: literal\nExecution halted\nsecond", "\tindent"):
        for reference in (False, True):
            footer = "Execution halted\n" if reference else ""
            text = "ABError: " + message + "\n" + footer
            assert stream_matches(text, "AB", message, error=True, reference=reference)
            mutations = (
                text.replace("AB", "BA", 1),
                text[1:],
                "X" + text,
                text.replace("AB", "", 1),
                text.replace("Error: ", "Error: changed", 1),
                text.replace(message, message[:-1], 1) if message else text.replace("Error: ", "Error: x", 1),
                text + "extra\n",
                "Error: " + message + "\nAB" + footer,
                "ABError: " + message[::-1] + "\n" + footer if len(message) > 1 else text + "x",
            )
            for changed in mutations:
                assert not stream_matches(changed, "AB", message, error=True, reference=reference), repr(changed)
    assert not stream_matches(
        "ABError: first\nsecond\n", "AB", "first\nExecution halted\nsecond", error=True, reference=False
    )
    assert not stream_matches("ABError in f() : extra : \n", "AB", "", error=True, reference=False)
    assert stream_matches("ABinteger:42\n", "ABinteger:42\n", "", error=False, reference=False)
    assert not stream_matches("BAinteger:42\n", "ABinteger:42\n", "", error=False, reference=False)
