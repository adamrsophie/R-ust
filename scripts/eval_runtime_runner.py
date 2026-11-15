"""All 74 rows in all five CLI modes, preserving separate and merged streams."""
from __future__ import annotations

import json
from pathlib import Path
from typing import TypedDict

from dynamic_frame_capture import capture, invocation, runtime_hashes
from dynamic_frame_records import Capture
from eval_runtime_compare import classification, exact
from eval_runtime_records import FIXTURE_SHA, MODES, Row
from eval_runtime_reference import read_reference
from eval_runtime_session import Session, create, hashes, inputs


class ModeResult(TypedDict):
    mode: str
    reference_pass: bool
    native_result: str
    reference: Capture
    native: Capture


class RowResult(TypedDict):
    id: str
    modes: list[ModeResult]


def check_row(row: Row, session: Session) -> RowResult:
    source = session.output / "sources" / (row.case.identifier + ".R")
    modes: list[ModeResult] = []
    for mode in MODES:
        directory = session.output / "cases" / row.case.identifier / mode
        reference = capture(invocation(mode, source, session.reference), session.reference, directory / "reference")
        native = capture(invocation(mode, source, session.native), session.native, directory / "native")
        modes.append(ModeResult(mode=mode, reference_pass=exact(row.raw, reference),
                                native_result=classification(row, native), reference=reference, native=native))
    result = RowResult(id=row.case.identifier, modes=modes)
    _ = (session.output / "cases" / row.case.identifier / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def run(binary: Path, output: Path, action: str) -> int:
    rows = read_reference()
    output.mkdir(parents=True, exist_ok=False)
    sources = inputs()
    executables = binary, binary.parent / "Rscript"
    before = hashes((*sources, *executables))
    _ = (output / "inputs-before.json").write_text(json.dumps(before, indent=2) + "\n")
    results: list[RowResult] = []
    session = create(binary, output)
    (output / "sources").mkdir()
    for row in rows:
        _ = (output / "sources" / (row.case.identifier + ".R")).write_bytes(row.case.source)
    programs = tuple(sorted((output / "sources").iterdir()))
    snapshots = (output / "bin/r_ust_cli", output / "bin/Rscript")
    captured_before = hashes((*programs, *snapshots))
    _ = (output / "capture-inputs-before.json").write_text(json.dumps(captured_before, indent=2) + "\n")
    complete = False
    try:
        for row in rows:
            result = check_row(row, session)
            results.append(result)
            passed = sum(mode["native_result"] == "CLI_PASS" for mode in result["modes"])
            print(f"{row.case.identifier}: {passed}/5 CLI", flush=True)
        complete = True
    finally:
        after = hashes((*sources, *executables))
        captured_after = hashes((*programs, *snapshots))
        runtime_after = runtime_hashes(session.reference.executables, session.reference.environment)
        stable = before == after and sources == inputs() and captured_before == captured_after
        stable = stable and runtime_after == session.reference.hashes
        modes = [mode for row in results for mode in row["modes"]]
        reference_passes = sum(mode["reference_pass"] for mode in modes)
        native_passes = sum(mode["native_result"] == "CLI_PASS" for mode in modes)
        limits = sum(mode["native_result"] == "LIMIT" for mode in modes)
        passed = complete and stable and len(modes) == reference_passes == native_passes == 370
        report = dict(action=action, result="CLI_PASS_TYPED_SDK_REQUIRED" if passed else "LIMIT" if limits else "FAIL",
            complete=complete, stable=stable, comparisons=len(modes), required_comparisons=370,
            reference_passes=reference_passes, native_passes=native_passes, native_limits=limits,
            fixture_sha256=FIXTURE_SHA, typed_sdk_status="NOT_RUN_BY_CLI_VERIFIER", excluded_ids=[],
            accounting=[dict(id=row.case.identifier, cli_modes=5, sdk="typed-value" if row.typed else "terminal-error") for row in rows],
            inputs_before=before, inputs_after=after, capture_inputs_before=captured_before,
            capture_inputs_after=captured_after, runtime_before=session.reference.hashes, runtime_after=runtime_after,
            limits=["CLI compares formatted values and visible output; 74 SDK outcomes independently verify hidden type/name/class distinctions.",
                    "Compilation binding of the supplied executable to this checkout is a separate integration gate.",
                    "Only exact hexadecimal environment pointer tokens and explicit per-row native diagnostic envelopes differ."], cases=results)
        _ = (output / "verification.json").write_text(json.dumps(report, indent=2) + "\n")
        files = tuple(sorted(path for path in output.rglob("*") if path.is_file()))
        _ = (output / "output-manifest.json").write_text(json.dumps(hashes(files), indent=2) + "\n")
    assert complete and stable and reference_passes == 370, "incomplete/unstable capture or fresh R drift"
    return int(action == "compare" and not passed)
