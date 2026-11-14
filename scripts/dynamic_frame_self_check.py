"""Production-reader and comparator adversaries; no native parity claim.

Run: python3 scripts/check-dynamic-frames.py self-check
"""
from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import tempfile

from dynamic_frame_compare import (
    CONDITIONS, Conditions, Frame, expected_native, native_matches, raw_matches, read_conditions,
)
from dynamic_frame_records import Capture, Invocation, Row, unhex
from dynamic_frame_reference import FIXTURES, PINS, loader_rejects, read_reference, replace_field


def mutated_capture(gold: Capture, condition: Conditions) -> int:
    native = expected_native(gold, condition)
    assert native_matches(gold, native, condition)
    count = 0
    for field in ("stdout", "stderr", "merged"):
        value = unhex(native[field])
        for changed in (b"extra" + value, value + b"extra", value[::-1] if value else b"changed"):
            mutant = native.copy()
            mutant[field] = changed.hex()
            if mutant[field] != native[field]:
                assert not native_matches(gold, mutant, condition), (field, changed)
                count += 1
    mutant = native.copy()
    mutant["status"] = 1 - native["status"]
    assert not native_matches(gold, mutant, condition)
    return count + 1


def hostile_payloads() -> int:
    count = 0
    for payload in ("Calls: literal", "Error: literal\nExecution halted", "first\nsecond", "at 0xabc", ""):
        condition = Conditions("synthetic", "", (("warning", "ordered warning"), ("error", payload)), Frame.DOMAIN)
        gold = Capture(stdout=b"AB".hex(), stderr=b"frozen diagnostic".hex(),
                       merged=b"ABfrozen diagnostic".hex(), status=1,
                       invocation=Invocation(argv=["R"], stdin=None))
        native = expected_native(gold, condition)
        assert native_matches(gold, native, condition)
        for changed in (
            replace(condition, ordered=(("warning", "different"), ("error", payload))),
            replace(condition, ordered=(("error", payload + "changed"),)),
            replace(condition, frame=Frame.TYPE),
        ):
            assert not native_matches(gold, native, changed)
            count += 1
        for text in (b"BA" + unhex(native["stderr"]), unhex(native["stderr"]) + b"AB"):
            mutant = native.copy()
            mutant["merged"] = text.hex()
            assert not native_matches(gold, mutant, condition)
            count += 1
        count += mutated_capture(gold, condition)
    return count


def pointer_checks() -> None:
    gold = Capture(stdout=b"<environment: 0xabc> at 0xabc".hex(), stderr="",
                   merged=b"<environment: 0xabc> at 0xabc".hex(), status=0,
                   invocation=Invocation(argv=["R"], stdin=None))
    pointer = gold.copy()
    for field in ("stdout", "merged"):
        pointer[field] = unhex(gold[field]).replace(b"<environment: 0xabc>", b"<environment: 0x123>").hex()
    assert raw_matches(gold, pointer)
    for field in ("stdout", "merged"):
        unrelated = pointer.copy()
        unrelated[field] = unhex(pointer[field]).replace(b"at 0xabc", b"at 0x123").hex()
        assert not raw_matches(gold, unrelated)


def closure_checks(rows: tuple[Row, ...], conditions: dict[str, Conditions]) -> list[str]:
    cases = (("closure_global", b"x + 1L", b"x + 2L", "closure-body"),
             ("closure_nested", b"$a", b"$z", "list-name"),
             ("closure_global", b"x = 1L", b"z = 1L", "formal-tag"),
             ("body_nonforcing", b"UNFORCED", b"EXECUTED", "body-literal"))
    results: list[str] = []
    for identifier, old, new, label in cases:
        row = next(row for row in rows if row["id"] == identifier)
        gold = row["raw"]["run-1"]["stdin"]
        native = expected_native(gold, conditions[identifier])
        for field in ("stdout", "merged"):
            value = unhex(native[field])
            assert old in value
            native[field] = value.replace(old, new, 1).hex()
        assert not native_matches(gold, native, conditions[identifier]), label
        results.append(label)
    return results


def conditions_rejected(rows: tuple[Row, ...], path: Path = CONDITIONS) -> None:
    try:
        _ = read_conditions(rows, path)
    except AssertionError:
        return
    raise AssertionError("production condition reader accepted corruption")


def self_check() -> None:
    rows = read_reference() + read_reference("dynamic-frame-source-r-4.6.0.tsv")
    conditions = read_conditions(rows)
    checks = 0
    for row in rows:
        if row["contract"] == "pending-bytecode":
            continue
        for mode, gold in row["raw"]["run-1"].items():
            if mode == "typed":
                continue
            assert raw_matches(gold, row["raw"]["run-2"][mode])
            checks += mutated_capture(gold, conditions[row["id"]])
    reader_mutations: list[str] = []
    with tempfile.TemporaryDirectory(prefix="dynamic-frame-comparator-") as directory:
        scratch = Path(directory)
        for name in PINS:
            data = (FIXTURES / name).read_bytes()
            lines = data.splitlines(keepends=True)
            for column, label in ((4, "source"), (7, "closure-body-name-tag"), (6, "raw-warning-effect-order")):
                changed = replace_field(lines, column, unhex(lines[1].decode().split("\t")[column]) + b"changed")
                loader_rejects(name, changed, scratch)
                reader_mutations.append(name + ":" + label)
        condition_copy = scratch / "conditions.tsv"
        for label, changed in (("payload", CONDITIONS.read_bytes() + b"changed\n"),
                               ("missing", b"\n".join(CONDITIONS.read_bytes().splitlines()[:-1]) + b"\n")):
            _ = condition_copy.write_bytes(changed)
            conditions_rejected(rows, condition_copy)
            reader_mutations.append("condition:" + label)
        changed_row = rows[0].copy()
        changed_row["source"] += b"changed"
        conditions_rejected((changed_row, *rows[1:]))
        reader_mutations.append("condition:source-binding")
    pointer_checks()
    print(json.dumps(dict(result="PASS", original=184, supplemental=30,
        native_comparator_mutations=checks, hostile_payload_mutations=hostile_payloads(),
        production_reader_rejections=reader_mutations, closure_mutations=closure_checks(rows, conditions), pointer_only="accepted", unrelated_hex="rejected"), sort_keys=True))
