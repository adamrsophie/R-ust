from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Final

from scripts.constructor_reference import ROOT, Case, decode_hex
from scripts.r_value_literal import literal
from scripts.stop_capture import digest

DISPLAY_FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/primitive-handle-display-r-4.6.0.tsv"
DISPLAY_SHA: Final = "dec4bc86c3ad5d61a55d2ac1ebb8f859e479c0e9a8caa1d27e7f87e55db6172b"
DISPLAY_HEADER: Final = "source\tkind\tclass\tis_function\tlength\tvisible\tdisplay_hex"
HANDLE_SOURCES: Final = ("sum", "length", "invisible", "`return`", "missing")


@dataclass(frozen=True, slots=True)
class Handle:
    source: str
    kind: str
    class_name: str
    is_function: str
    length: int
    visible: str
    display: str


@dataclass(frozen=True, slots=True)
class PrimitiveCase(Case):
    visible: str
    source_stdout: str
    handle: Handle | None


def parse_metadata(text: str) -> tuple[Handle, ...]:
    assert "\r" not in text and "\0" not in text and text.endswith("\n")
    lines = text.splitlines()
    assert len(lines) == 6 and lines[0] == DISPLAY_HEADER
    handles: list[Handle] = []
    for identifier, line in enumerate(lines[1:]):
        source, kind, class_name, function, count, visible, display_hex = line.split("\t")
        assert source == HANDLE_SOURCES[identifier]
        assert kind == ("builtin" if identifier < 3 else "special")
        assert class_name == "function" and function == visible == "TRUE" and count == "1"
        display = decode_hex(display_hex)
        assert display.endswith("\n") and display.count("\n") == 1 and "\r" not in display
        assert display.endswith('.Primitive("' + source.strip("`") + '")\n')
        assert display.startswith(".Primitive(" if source == "`return`" else "function (")
        handles.append(Handle(source, kind, class_name, function, int(count), visible, display))
    return tuple(handles)


def read_metadata() -> tuple[Handle, ...]:
    assert digest(DISPLAY_FIXTURE) == DISPLAY_SHA
    return parse_metadata(DISPLAY_FIXTURE.read_text())


def program(row: PrimitiveCase) -> str:
    if row.error:
        return "options(warn=1);" + row.source + ';cat("AFTER\\n")\n'
    capture = "options(warn=1);.primitive_answer<-withVisible({" + row.source + "});"
    answer = ".primitive_answer$value"
    if row.handle is None:
        condition = f"identical({answer},{literal(row.fingerprint)})"
    else:
        handle = row.handle
        capture += f"print({answer});"
        condition = (
            f"identical(typeof({answer}),{json.dumps(handle.kind)})"
            + f" && identical(class({answer}),{json.dumps(handle.class_name)})"
            + f" && identical(is.function({answer}),{handle.is_function})"
            + f" && identical(length({answer}),{handle.length}L)"
        )
    condition += f" && identical(.primitive_answer$visible,{row.visible})"
    return capture + "cat(" + condition + ',"\\n",sep="")\n'


def self_check() -> None:
    metadata = read_metadata()
    text = DISPLAY_FIXTURE.read_text()
    assert len({handle.display for handle in metadata}) == 5
    lines = text.splitlines()
    for column, replacement in (
        (0, "length"),
        (1, "special"),
        (2, "builtin"),
        (3, "FALSE"),
        (4, "01"),
        (4, "2"),
        (5, "FALSE"),
        (6, "-"),
        (6, "ff"),
        (6, "0A"),
        (6, "0"),
        (6, ""),
        (6, b'.Primitive("sum")\n'.hex()),
        (6, metadata[1].display.encode().hex()),
        (6, (metadata[0].display + "extra\n").encode().hex()),
    ):
        fields = lines[1].split("\t")
        fields[column] = replacement
        changed = "\n".join((lines[0], "\t".join(fields), *lines[2:])) + "\n"
        try:
            _ = parse_metadata(changed)
        except (AssertionError, ValueError):
            continue
        raise AssertionError((column, replacement))
    for invalid in (
        "",
        text.rstrip("\n"),
        text.replace(DISPLAY_HEADER, "invalid", 1),
        text + lines[1] + "\n",
        "\n".join(lines[:-1]) + "\n",
        text.replace("\n", "\r\n"),
        text.replace(lines[2], lines[1], 1),
    ):
        try:
            _ = parse_metadata(invalid)
        except (AssertionError, ValueError):
            continue
        raise AssertionError("invalid metadata accepted")
