"""Adversarial reader/comparator checks; no R or native process is executed."""
from __future__ import annotations

from collections.abc import Callable
from dataclasses import replace
import json
from typing import Literal

from dynamic_frame_records import Capture, normalized, unhex
from eval_runtime_compare import FRAMES, classification, expected_native, native_matches
from eval_runtime_nodes import node
from eval_runtime_records import CASES, ERRORS, FIXTURE, HEADER, ROOT, Row
from eval_runtime_reference import cases, parse, read_data, read_reference
from eval_runtime_session import profile


def rejected(action: Callable[[], None]) -> None:
    try:
        action()
    except (AssertionError, ValueError, TypeError, IndexError):
        return
    raise AssertionError("mutated input was accepted")


def changed(data: bytes, field: str, value: str) -> bytes:
    lines = data.decode().splitlines()
    fields = lines[1].split("\t")
    fields[HEADER.index(field)] = value
    lines[1] = "\t".join(fields)
    return ("\n".join(lines) + "\n").encode()


def reader_checks() -> list[str]:
    data, source, errors = FIXTURE.read_bytes(), CASES.read_bytes(), ERRORS.read_bytes()
    inputs = cases(source)
    checked: list[str] = []
    mutations = {
        "truncated-final-newline": data[:-1], "extra-column": data.replace(b"\tidentity\t", b"\tidentity\textra\t", 1),
        "bad-header": data.replace(b"node_hex", b"node", 1),
        "missing-row": b"\n".join(data.split(b"\n")[:-2]) + b"\n",
        "duplicate-row": data.splitlines(True)[0] + data.splitlines(True)[1] + b"".join(data.splitlines(True)[1:]),
        "malformed-hex": changed(data, "node_hex", "0A"),
        "wrong-type": changed(data, "typeof", "integer"),
        "wrong-class": changed(data, "class_hex", b"invented".hex()),
        "wrong-mode": changed(data, "mode", "numeric"),
        "wrong-length": changed(data, "length", "2"),
        "bad-visibility": changed(data, "visible", "true"),
        "wrong-error-status": changed(data, "status", "1"),
        "spurious-warning": changed(data, "stderr_hex", b"warning\n".hex()),
    }
    for name, candidate in mutations.items():
        def attempt() -> None:
            _ = parse(candidate, inputs, errors)
        rejected(attempt)
        checked.append(name)
    for field, value in (("node_hex", b'["string","78","unknown"]'.hex()),
                         ("visible", "FALSE"), ("print_hex", b"changed".hex()),
                         ("effects_hex", b"EXTRA".hex()), ("merged_hex", b"EXTRA".hex())):
        def frozen() -> None:
            _ = read_data(changed(data, field, value), source, errors)
        rejected(frozen)
        checked.append("immutable-" + field)
    for candidate in (source.replace(b"6576616c", b"65766171", 1), source[:-1], source + source.splitlines(True)[1]):
        def source_changed() -> None:
            _ = read_data(data, candidate, errors)
        rejected(source_changed)
        checked.append("immutable-source")
    duplicate = source.splitlines(True)
    duplicate[2] = duplicate[1]
    def duplicate_case() -> None:
        _ = cases(b"".join(duplicate))
    rejected(duplicate_case)
    checked.append("duplicate-case")
    for candidate in (errors[:-1], errors.replace(b"65787072", b"65787073", 1)):
        def error_changed() -> None:
            _ = read_data(data, source, candidate)
        rejected(error_changed)
        checked.append("immutable-error-payload")
    return checked


def node_checks() -> list[str]:
    malformed = ('[]', '["mystery"]', '["nil",1]', '["integer","2147483648"]',
        '["integer","01"]', '["real","00"]', '["string","0g","unknown"]',
        '["symbol",""]', '["primitive","eval"]', '["pairlist",[]]',
        '["pairlist",[["",["nil"]]]]', '["vector","integer",[["string","61","unknown"]]]',
        '["vector","integer",[["vector","integer",[["integer","1"]]]]]')
    from eval_runtime_records import node_json
    for encoded in malformed:
        def attempt() -> None:
            _ = node(node_json(encoded.encode()))
        rejected(attempt)
    return ["node-" + str(index) for index in range(len(malformed))]


def altered(gold: Capture, field: Literal["stdout", "stderr", "merged"], value: bytes) -> Capture:
    result = gold.copy()
    result[field] = value.hex()
    return result


def comparator_checks(rows: tuple[Row, ...]) -> list[str]:
    checked: list[str] = []
    assert set(FRAMES) == {row.case.identifier for row in rows if row.error is not None}
    for row in rows:
        gold = expected_native(row)
        assert native_matches(row, gold)
        status = gold.copy()
        status["status"] = 1 - gold["status"]
        assert not native_matches(row, status)
        for field in ("stdout", "stderr", "merged"):
            raw = unhex(gold[field])
            for mutation in (b"JUNK" + raw, raw + b"JUNK", raw[:-1] if raw else b"X"):
                assert not native_matches(row, altered(gold, field, mutation)), (row.case.identifier, field)
        if row.typed is not None and row.typed.warnings:
            assert gold["stderr"] == row.raw["stderr"]
            assert not native_matches(row, altered(gold, "stderr", b""))
            legacy = ("Warning: " + row.typed.warnings[0].message + "\n").encode()
            assert not native_matches(row, altered(gold, "stderr", legacy))
        if row.error is not None:
            for payload in (b"Calls: forged\n", b"Execution halted\n", b"Error in user_call() : changed\n"):
                assert not native_matches(row, altered(gold, "stderr", unhex(gold["stderr"]) + payload))
            assert not native_matches(row, altered(gold, "stderr", b"type error: " + row.error.encode() + b"\n"))
        if gold["stderr"] and gold["stdout"]:
            wrong = unhex(gold["stderr"]) + unhex(gold["stdout"])
            if wrong == unhex(gold["merged"]):
                wrong = unhex(gold["stdout"]) + unhex(gold["stderr"])
            assert not native_matches(row, altered(gold, "merged", wrong))
        limited = altered(gold, "stderr", b"eval is not implemented yet\n")
        assert classification(row, limited) == "LIMIT" and not native_matches(row, limited)
        checked.append(row.case.identifier)
    assert normalized(b"<environment: 0x123>") == normalized(b"<environment: 0xabc>")
    for left, right in ((b"0x123", b"0xabc"), (b"<environment:0x123>", b"<environment:0xabc>"),
                        (b"<bytecode: 0x123>", b"<bytecode: 0xabc>")):
        assert normalized(left) != normalized(right)
    return checked


def literal_envelope_checks(rows: tuple[Row, ...]) -> list[str]:
    row = next(row for row in rows if row.case.identifier == "expression_error")
    actual = row.raw.copy()
    actual.update(stdout=b"FIRST\n".hex(), stderr=b"sequence failure\n".hex(),
                  merged=b"FIRST\nsequence failure\n".hex())
    assert native_matches(row, actual)
    payload = "first\nCalls: authored_user_text()\nExecution halted\nlast"
    reference = row.raw.copy()
    diagnostic = ("Error in eval() : " + payload + "\nCalls: eval -> eval\nExecution halted\n").encode()
    reference.update(stderr=diagnostic.hex(), merged=(b"FIRST\n" + diagnostic).hex())
    synthetic = replace(row, error=payload, raw=reference)
    actual.update(stderr=(payload + "\n").encode().hex(), merged=("FIRST\n" + payload + "\n").encode().hex())
    assert native_matches(synthetic, actual)
    for changed_payload in (payload.replace("Calls: authored_user_text()\n", ""),
                            payload.replace("authored_user_text", "wrong_call"),
                            payload.replace("Execution halted\n", ""), payload + "\ntrailing"):
        changed_actual = actual.copy()
        changed_actual.update(stderr=(changed_payload + "\n").encode().hex(),
                              merged=("FIRST\n" + changed_payload + "\n").encode().hex())
        assert not native_matches(synthetic, changed_actual)
    return ["literal-sequence-error", "calls-looking-payload-preserved", "changed-call-like-text",
            "execution-halted-looking-payload-preserved", "trailing-payload-junk"]


def self_check() -> None:
    rows = read_reference()
    _ = profile()
    readers = reader_checks()
    nodes = node_checks()
    comparisons = comparator_checks(rows)
    envelopes = literal_envelope_checks(rows)
    from eval_runtime_runner import run
    try:
        _ = run(ROOT / "never-executed", ROOT, "compare")
    except FileExistsError:
        envelopes.append("existing-output-refused-before-runtime")
    else:
        raise AssertionError("existing output accepted")
    print(json.dumps(dict(result="SELF_CHECK_PASS", reader_scenarios=readers,
          node_scenarios=nodes, comparator_cases=comparisons, rows=74, typed=52, errors=22,
          cli_mode_pairs_required=370, native_executions=0, reference_executions=0,
          literal_envelope_scenarios=envelopes), indent=2))
