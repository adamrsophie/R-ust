#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run self-check, baseline BINARY NEW_OUTPUT, or compare BINARY NEW_OUTPUT."""
from __future__ import annotations

import sys

from dataclasses import dataclass
import json
from pathlib import Path
import shutil
from typing import TypedDict

from dynamic_frame_capture import Runtime, capture, discover, invocation, runtime_hashes, write_json
from dynamic_frame_compare import Conditions, native_matches, raw_matches, read_conditions
from dynamic_frame_records import Capture, Contract, Row, Scope, sha
from dynamic_frame_reference import ROOT, read_reference
from dynamic_frame_self_check import self_check

MODES = ("expression", "file", "stdin", "rscript-file", "rscript-expression")


class ModeResult(TypedDict):
    mode: str
    reference_pass: bool
    native_pass: bool
    reference: Capture
    native: Capture


class RowResult(TypedDict):
    id: str
    modes: list[ModeResult]
    reference_passes: int
    native_passes: int


class Group(TypedDict):
    comparisons: int
    native_passes: int
    reference_passes: int
    cases: list[RowResult]


class Accounting(TypedDict):
    id: str
    scope: Scope
    contract: Contract
    native_cli: bool
    separate_proof_required: str


class Verification(TypedDict):
    action: str
    parity: bool
    stable: bool
    groups: dict[str, Group]
    accounting: list[Accounting]
    inputs_before: dict[str, str]
    inputs_after: dict[str, str]
    capture_inputs_before: dict[str, str]
    capture_inputs_after: dict[str, str]
    runtime_before: dict[str, str]
    runtime_after: dict[str, str]
    limits: list[str]


def write_report(path: Path, value: RowResult | Verification) -> None:
    _ = path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


@dataclass(frozen=True, slots=True)
class Session:
    output: Path
    reference: Runtime
    native: Runtime


def source_inputs() -> tuple[Path, ...]:
    return tuple(sorted({*ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"),
        *ROOT.glob("crates/**/Cargo.toml"), ROOT / "Cargo.toml", ROOT / "Cargo.lock",
        *ROOT.glob("scripts/dynamic_frame*"), ROOT / "scripts/dynamic-frame-conditions.tsv", Path(__file__).resolve()}))


def hashes(paths: tuple[Path, ...]) -> dict[str, str]:
    return {str(path): sha(path.read_bytes()) for path in paths}


def check_row(row: Row, condition: Conditions, session: Session) -> RowResult:
    identifier = row["id"]
    source = session.output / "sources" / (identifier + ".R")
    results: list[ModeResult] = []
    for mode in MODES:
        directory = session.output / "cases" / identifier / mode
        reference = capture(invocation(mode, source, session.reference), session.reference, directory / "reference")
        gold = row["raw"]["run-1"][mode]
        fresh = raw_matches(gold, reference)
        actual = capture(invocation(mode, source, session.native), session.native, directory / "native")
        passed = fresh and native_matches(gold, actual, condition)
        results.append(ModeResult(mode=mode, reference_pass=fresh, native_pass=passed,
                            reference=reference, native=actual))
    return RowResult(id=identifier, modes=results,
        reference_passes=sum(result["reference_pass"] is True for result in results),
        native_passes=sum(result["native_pass"] is True for result in results))


def accounting(row: Row) -> Accounting:
    private = {
        "sdk_evalq_same_environment": "selected_return_context_counts_as_function",
        "sdk_evalq_break_same_environment": "same_environment_eval_loop_target_survives_return_context",
        "sdk_evalq_next_same_environment": "same_environment_eval_loop_target_survives_return_context",
        "sdk_evalq_return_boundary": "selected_return_catches_own_return",
    }
    return Accounting(id=row["id"], scope=row["scope"], contract=row["contract"],
        native_cli=row["scope"] == "cli" and row["contract"] == "supported",
        separate_proof_required=private.get(row["id"], ""))


def compare(binary: Path, output: Path, action: str) -> bool:
    output.mkdir(parents=True, exist_ok=False)
    original = read_reference()
    supplemental = read_reference("dynamic-frame-source-r-4.6.0.tsv")
    rows = original + supplemental
    conditions = read_conditions(rows)
    sources = source_inputs()
    binaries = binary, binary.parent / "Rscript"
    before = hashes((*sources, *binaries))
    write_json(output / "inputs-before.json", before)
    reference = discover(original[0]["profile"], output / "reference-runtime")
    (output / "bin").mkdir()
    snapshots = output / "bin/r_ust_cli", output / "bin/Rscript"
    for src, dest in zip(binaries, snapshots, strict=True):
        _ = shutil.copy2(src, dest)
        assert sha(src.read_bytes()) == sha(dest.read_bytes())
    native = Runtime((str(snapshots[0]), str(snapshots[1])), reference.environment, hashes(snapshots))
    selected = tuple(row for row in rows if row["scope"] == "cli" and row["contract"] == "supported")
    (output / "sources").mkdir()
    for row in selected:
        _ = (output / "sources" / (row["id"] + ".R")).write_bytes(row["source"])
    programs = tuple(sorted((output / "sources").iterdir()))
    captured_before = hashes((*programs, *snapshots))
    write_json(output / "capture-inputs-before.json", captured_before)
    session = Session(output, reference, native)
    groups: dict[str, Group] = {}
    for name, group in (("original", original), ("supplemental", supplemental)):
        current: list[RowResult] = []
        for row in group:
            if row not in selected:
                continue
            result = check_row(row, conditions[row["id"]], session)
            current.append(result)
            write_report(output / "cases" / row["id"] / "result.json", result)
            print(f"{name}/{row['id']}: {result['native_passes']}/5 native; {result['reference_passes']}/5 R", flush=True)
        native_passes = sum(item["native_passes"] for item in current)
        reference_passes = sum(item["reference_passes"] for item in current)
        groups[name] = Group(comparisons=5 * len(current), native_passes=native_passes,
                           reference_passes=reference_passes, cases=current)
    after = hashes((*sources, *binaries))
    captured_after = hashes((*programs, *snapshots))
    reference_after = runtime_hashes(reference.executables, reference.environment)
    stable = before == after and captured_before == captured_after and sources == source_inputs()
    stable = stable and reference.hashes == reference_after
    passed = stable and all(item["native_passes"] == item["comparisons"] for item in groups.values())
    write_report(output / "verification.json", Verification(action=action, parity=passed, stable=stable, groups=groups,
        accounting=[accounting(row) for row in rows],
        inputs_before=before, inputs_after=after, capture_inputs_before=captured_before,
        capture_inputs_after=captured_after, runtime_before=reference.hashes, runtime_after=reference_after,
        limits=["Capture stability does not establish compilation binding to the supplied binary.",
                "Four original SDK/reference-only evalq rows require private native context tests.",
                "The supplemental uncompiled namespace closure is SDK-only; no setter-program rewrite is executed.",
                "Three supplemental bytecode cases remain pending without native execution or pointer masking."] ))
    raw_files = tuple(sorted(path for path in output.rglob("*") if path.is_file()))
    write_json(output / "output-manifest.json", hashes(raw_files))
    assert stable, "input membership or bytes changed during capture"
    assert all(item["reference_passes"] == item["comparisons"] for item in groups.values()), "fresh R differs"
    return passed


def main() -> int:
    usage = "self-check | baseline BINARY NEW_OUTPUT | compare BINARY NEW_OUTPUT"
    assert len(sys.argv) in (2, 4), usage
    action = sys.argv[1]
    if action == "self-check":
        assert len(sys.argv) == 2, usage
        self_check()
        return 0
    assert action in ("baseline", "compare") and len(sys.argv) == 4, usage
    passed = compare(Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve(), action)
    return int(action == "compare" and not passed)


if __name__ == "__main__":
    raise SystemExit(main())
