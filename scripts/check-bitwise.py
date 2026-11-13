#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/check-bitwise.py [compare|baseline|self-check] [binary] [output-directory]
from __future__ import annotations

import json
import shutil
import sys
from dataclasses import asdict, dataclass
from enum import StrEnum
from pathlib import Path
from typing import assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.bitwise_reference import (
    FIXTURE_SHA,
    BitwiseCase,
    matches,
    program,
    read_cases,
    self_check,
)
from scripts.constructor_reference import ROOT
from scripts.stop_capture import MODES, Mode, Run, digest, encode_bytes, invocation, run
from scripts.stop_reference import live_profile


class Action(StrEnum):
    BASELINE = "baseline"
    COMPARE = "compare"
    SELF_CHECK = "self-check"


@dataclass(frozen=True, slots=True)
class Comparison:
    mode: Mode
    case: BitwiseCase
    r: Run
    native: Run
    r_merged: Run
    native_merged: Run
    passed: bool
    exact_streams: bool
    exact_merged: bool


def source_inputs() -> tuple[Path, ...]:
    return tuple(
        sorted(
            (
                *ROOT.glob("crates/**/*.rs"),
                *ROOT.glob("crates/**/*.tsv"),
                *ROOT.glob("crates/**/Cargo.toml"),
                ROOT / "Cargo.toml",
                ROOT / "Cargo.lock",
                Path(__file__),
                ROOT / "scripts/bitwise_reference.py",
                ROOT / "scripts/bitwise-oracle.R",
                ROOT / "scripts/constructor_reference.py",
                ROOT / "scripts/r_value_literal.py",
                ROOT / "scripts/stop_capture.py",
                ROOT / "scripts/stop_reference.py",
            )
        )
    )


def compare(binary: Path, directory: Path, *, baseline: bool) -> int:
    self_check()
    profiles = live_profile()
    cases = read_cases()
    inputs = source_inputs()
    sources = {str(path): digest(path) for path in inputs}
    references = {
        str(Path(shutil.which(name) or name).resolve()): digest(Path(shutil.which(name) or name).resolve())
        for name in ("R", "Rscript")
    }
    originals = (binary, binary.parent / "Rscript")
    hashes = {str(path): digest(path) for path in originals}
    directory.mkdir(parents=True, exist_ok=True)
    snapshots = tuple(directory / "bin" / name for name in ("r_ust_cli", "Rscript"))
    snapshots[0].parent.mkdir(exist_ok=True)
    for original, snapshot in zip(originals, snapshots, strict=True):
        assert original != snapshot.resolve()
        _ = shutil.copy2(original, snapshot)
    assert all(digest(original) == digest(snapshot) for original, snapshot in zip(originals, snapshots, strict=True))
    source_directory = directory / "sources"
    source_directory.mkdir(exist_ok=True)
    programs = tuple(source_directory / f"{case.identifier:03d}.R" for case in cases)
    for case, source in zip(cases, programs, strict=True):
        _ = source.write_text(program(case))
    program_hashes = {str(path): digest(path) for path in programs}
    results: list[Comparison] = []
    for mode in MODES:
        for case, source in zip(cases, programs, strict=True):
            reference = tuple(
                run(invocation(mode, ("R", "Rscript"), source), merged=merged) for merged in (False, True)
            )
            native = tuple(
                run(invocation(mode, (str(snapshots[0]), str(snapshots[1])), source), merged=merged)
                for merged in (False, True)
            )
            r, r_merged = reference
            actual, actual_merged = native
            assert matches(case, (r, r_merged), reference=True), (
                f"live R disagrees: {mode}/{case.identifier}: {reference}"
            )
            results.append(
                Comparison(
                    mode,
                    case,
                    r,
                    actual,
                    r_merged,
                    actual_merged,
                    matches(case, (actual, actual_merged), reference=False),
                    (r.stdout_base64, r.stderr_base64) == (actual.stdout_base64, actual.stderr_base64),
                    r_merged.stdout_base64 == actual_merged.stdout_base64,
                )
            )
        print(f"Checked {mode}", flush=True)
    assert [(row.mode, row.case.identifier) for row in results] == [
        (mode, case.identifier) for mode in MODES for case in cases
    ]
    assert len(results) == 1510
    binaries_stable = all(
        digest(original) == hashes[str(original)] == digest(snapshot)
        for original, snapshot in zip(originals, snapshots, strict=True)
    )
    source_stable = inputs == source_inputs() and all(digest(Path(path)) == value for path, value in sources.items())
    programs_stable = all(digest(Path(path)) == value for path, value in program_hashes.items())
    reference_stable = all(digest(Path(path)) == value for path, value in references.items())
    failures = [row for row in results if not row.passed]
    report = directory / "report.json"
    _ = report.write_text(
        json.dumps(
            {
                "fixture_sha256": FIXTURE_SHA,
                "supported_sources": 302,
                "expected_comparisons": 1510,
                "ordering_adapter": "Only exact source/ID-bound rows35,86,137,188,239 require their B effect prefix before warnings in both captures.",
                "reference_error_rendering": "For21 exact source/ID/error-bound newline-terminated messages, R uses the existing newline; immutable condition payloads remain unchanged. Native framing is not trimmed.",
                "reference_profile": profiles,
                "reference_sha256": references,
                "source_sha256": sources,
                "program_sha256": program_hashes,
                "binary_sha256": hashes,
                "baseline": baseline,
                "source_stable": source_stable,
                "programs_stable": programs_stable,
                "binaries_stable": binaries_stable,
                "reference_stable": reference_stable,
                "cases": [asdict(row) for row in results],
                "exact_success_comparisons": sum(
                    row.passed and not row.case.error and row.exact_streams and row.exact_merged for row in results
                ),
                "diagnostic_comparisons": sum(row.passed and bool(row.case.error) for row in results),
                "raw_stream_differences": sum(not row.exact_streams for row in results),
                "raw_merged_differences": sum(not row.exact_merged for row in results),
                "limits": [
                    "All 302 immutable fixture cases run across five entry modes; no cases are excluded.",
                    "Exact error/warning payloads, status, order and after-error sentinel absence are checked inside CLI framing.",
                    "Full integer values and recorded visibility use fixture-derived identical checks; each original source runs once.",
                    "R 4.6.0 aarch64 LDOUBLE8 JIT0 UTF-8 profile only; cases beyond the immutable fixture are not certified.",
                    "A baseline records an older executable against current source inputs; it is not a revision-bound success claim.",
                ],
            },
            default=encode_bytes,
            indent=2,
        )
        + "\n"
    )
    summary = [f"report\t{report}\t{digest(report)}"]
    summary.extend(
        f"input\t{path}\t{value}" for path, value in (sources | hashes | references | program_hashes).items()
    )
    summary.extend(f"row\t{row.mode}/{row.case.identifier}\t{'PASS' if row.passed else 'FAIL'}" for row in results)
    _ = (directory / "summary.tsv").write_text("\n".join(summary) + "\n")
    for row in failures:
        print(f"FAIL {row.mode}/{row.case.identifier}")
    stable = source_stable and programs_stable and reference_stable and binaries_stable
    print(f"{len(results) - len(failures)}/{len(results)} CLI comparisons passed; baseline={baseline}; stable={stable}")
    return int(not stable or (not baseline and bool(failures)))


def main() -> int:
    try:
        action = Action(sys.argv[1] if len(sys.argv) > 1 else "compare")
    except ValueError:
        raise SystemExit("Usage: check-bitwise.py [compare|baseline|self-check] [binary] [output-directory]") from None
    match action:
        case Action.SELF_CHECK:
            self_check()
            print(f"Comparator self-check passed; {len(read_cases())} independent fixture rows validated")
            return 0
        case Action.COMPARE | Action.BASELINE:
            binary = Path(sys.argv[2] if len(sys.argv) > 2 else ROOT / "target/debug/r_ust_cli").resolve()
            directory = Path(sys.argv[3]) if len(sys.argv) > 3 else ROOT / "target/bitwise-parity" / action.value
            return compare(binary, directory.resolve(), baseline=action == Action.BASELINE)
        case _:
            assert_never(action)


if __name__ == "__main__":
    raise SystemExit(main())
