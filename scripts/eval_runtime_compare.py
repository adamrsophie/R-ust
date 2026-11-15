"""Exact stream order with explicitly declared native diagnostic envelopes.

GNU R bytes are never line-stripped. Only the complete, frozen diagnostic block
is replaced with the corresponding source-declared native CLI representation.
This formatted CLI gate is separate from the exact typed SDK value gate.
"""
from __future__ import annotations

from typing import Final

from dynamic_frame_records import Capture, normalized, unhex
from eval_runtime_records import Row

FRAMES: Final = {
    "primitive_rejected": "domain error: ",
    "missing_expression": "", "missing_evalq": "", "computed_missing": "",
    "bound_missing": "", "omitted_forward": "", "expr_force_first": "",
    "environment_invalid_enclos": "domain error: ", "invalid_both": "domain error: ",
    "argument_order": "domain error: ", "evalq_force_order": "domain error: ",
    "logical_envir": "domain error: ", "character_envir": "domain error: ",
    "numeric_length": "domain error: ", "numeric_na": "domain error: ",
    "numeric_range": "domain error: ", "partial_escalated": "domain error: ",
    "unused_arg": "argument error: ", "ambiguous_arg": "argument error: ",
    "expression_error": "", "break_outside": "control-flow error: ",
    "break_other_environment": "control-flow error: ",
}


def exact(gold: Capture, actual: Capture) -> bool:
    return gold["status"] == actual["status"] and all(
        normalized(unhex(gold[key])) == normalized(unhex(actual[key]))
        for key in ("stdout", "stderr", "merged")
    )


def native_diagnostic(row: Row) -> bytes:
    if row.error is not None:
        assert row.case.identifier in FRAMES and row.typed is None
        return (FRAMES[row.case.identifier] + row.error + "\n").encode()
    assert row.case.identifier not in FRAMES and row.typed is not None
    return "".join("Warning: " + warning.message + "\n" for warning in row.typed.warnings).encode()


def expected_native(row: Row) -> Capture:
    result = row.raw.copy()
    stderr, merged = (unhex(row.raw[key]) for key in ("stderr", "merged"))
    replacement = native_diagnostic(row)
    if stderr:
        assert merged.count(stderr) == 1, "unmeasured split diagnostic sequence"
        assert merged.replace(stderr, b"", 1) == unhex(row.raw["stdout"]), "raw event order inconsistent"
        merged = merged.replace(stderr, replacement, 1)
    else:
        assert not replacement and merged == unhex(row.raw["stdout"])
    result["stderr"] = replacement.hex()
    result["merged"] = merged.hex()
    return result


def native_matches(row: Row, actual: Capture) -> bool:
    return exact(expected_native(row), actual)


def classification(row: Row, actual: Capture) -> str:
    if native_matches(row, actual):
        return "CLI_PASS"
    if b"is not implemented yet" in unhex(actual["stderr"]):
        return "LIMIT"
    return "FAIL"
