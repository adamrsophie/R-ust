#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# Run: uv run scripts/check-constructors.py [compare|baseline|self-check] [binary]
# Or: python3 scripts/check-constructors.py [compare|baseline|self-check] [binary]
from __future__ import annotations

import json
import shutil
import sys
import tempfile
from dataclasses import asdict, dataclass
from enum import StrEnum
from pathlib import Path
from typing import assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.constructor_reference import (
    FIXTURE_SHA,
    ROOT,
    SELECTED,
    Case,
    Lane,
    matches,
    program,
    read_cases,
    self_check,
)
from scripts.stop_capture import MODES, Mode, Run, digest, encode_bytes, invocation, run
from scripts.stop_reference import live_profile


class Action(StrEnum):
    BASELINE = "baseline"
    COMPARE = "compare"
    SELF_CHECK = "self-check"


@dataclass(frozen=True, slots=True)
class Comparison:
    mode: Mode
    case: Case
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
                ROOT / "scripts/constructor_reference.py",
                ROOT / "scripts/stop_capture.py",
                ROOT / "scripts/stop_reference.py",
            )
        )
    )


def compare(binary: Path, *, baseline: bool) -> int:
    self_check()
    profiles = live_profile()
    rows = read_cases()
    cases = tuple(row for row in rows if row.identifier in SELECTED)
    inputs = source_inputs()
    sources = {str(path): digest(path) for path in inputs}
    references = {
        str(Path(shutil.which(name) or name).resolve()): digest(Path(shutil.which(name) or name).resolve())
        for name in ("R", "Rscript")
    }
    originals = (binary, binary.parent / "Rscript")
    hashes = {str(path): digest(path) for path in originals}
    directory = ROOT / "target/constructor-parity" / ("baseline" if baseline else "compare")
    directory.mkdir(parents=True, exist_ok=True)
    snapshots = tuple(directory / "bin" / name for name in ("r_ust_cli", "Rscript"))
    snapshots[0].parent.mkdir(exist_ok=True)
    for original, snapshot in zip(originals, snapshots, strict=True):
        assert original != snapshot.resolve()
        _ = shutil.copy2(original, snapshot)
    assert all(digest(original) == digest(snapshot) for original, snapshot in zip(originals, snapshots, strict=True))
    results: list[Comparison] = []
    with tempfile.TemporaryDirectory(prefix="r-ust-constructors-") as temporary:
        source = Path(temporary) / "constructor source.R"
        for mode in MODES:
            for case in cases:
                _ = source.write_text(program(case))
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
    binaries_stable = all(
        digest(original) == hashes[str(original)] == digest(snapshot)
        for original, snapshot in zip(originals, snapshots, strict=True)
    )
    source_stable = inputs == source_inputs() and all(digest(Path(path)) == value for path, value in sources.items())
    reference_stable = all(digest(Path(path)) == value for path, value in references.items())
    failures = [row for row in results if not row.passed]
    report = directory / "report.json"
    _ = report.write_text(
        json.dumps(
            {
                "fixture_sha256": FIXTURE_SHA,
                "reference_profile": profiles,
                "reference_sha256": references,
                "source_sha256": sources,
                "binary_sha256": hashes,
                "baseline": baseline,
                "source_stable": source_stable,
                "binaries_stable": binaries_stable,
                "reference_stable": reference_stable,
                "selected_ids": SELECTED,
                "cases": [asdict(row) for row in results],
                "exact_success_comparisons": sum(
                    row.passed
                    and not row.case.error
                    and not row.case.warnings
                    and row.exact_streams
                    and row.exact_merged
                    for row in results
                ),
                "diagnostic_comparisons": sum(
                    row.passed and bool(row.case.error or row.case.warnings) for row in results
                ),
                "raw_stream_differences": sum(not row.exact_streams for row in results),
                "raw_merged_differences": sum(not row.exact_merged for row in results),
                "future_types": [asdict(row) for row in rows if row.lane == Lane.FUTURE],
                "limits": [
                    "628 represented fixture cases belong to the SDK suite; CLI selects 102 across five entry modes.",
                    "20 future-type fixture cases are retained as limitations, never counted as passing CLI checks.",
                    "Error headers and call stacks differ; exact payload, warning order, effects, status and raw streams retained.",
                    "Successful values use fixture-derived identical checks; general pretty-printing is not certified.",
                    "warn=1 exposes warning order; explicit warn=2 fixture settings still take precedence.",
                    "R 4.6.0 aarch64 LDOUBLE8 JIT0 profile only; no huge valid allocation probes.",
                ],
            },
            default=encode_bytes,
            indent=2,
        )
        + "\n"
    )
    summary = [f"report\t{report}\t{digest(report)}"]
    summary.extend(f"input\t{path}\t{value}" for path, value in (sources | hashes | references).items())
    summary.extend(f"row\t{row.mode}/{row.case.identifier}\t{'PASS' if row.passed else 'FAIL'}" for row in results)
    summary.extend(f"future-type\t{row.identifier}\tLIMITED" for row in rows if row.lane == Lane.FUTURE)
    _ = (directory / "summary.tsv").write_text("\n".join(summary) + "\n")
    for row in failures:
        print(f"FAIL {row.mode}/{row.case.identifier}")
    stable = source_stable and reference_stable and binaries_stable
    print(f"{len(results) - len(failures)}/{len(results)} CLI comparisons passed; stable={stable}")
    return 0 if baseline else int(bool(failures) or not stable)


def main() -> int:
    try:
        action = Action(sys.argv[1] if len(sys.argv) > 1 else "compare")
    except ValueError:
        raise SystemExit("Usage: check-constructors.py [compare|baseline|self-check] [binary]") from None
    match action:
        case Action.SELF_CHECK:
            self_check()
            print(f"Comparator self-check passed; {len(read_cases())} independent fixture rows validated")
            return 0
        case Action.COMPARE | Action.BASELINE:
            binary = Path(sys.argv[2] if len(sys.argv) > 2 else ROOT / "target/debug/r_ust_cli").resolve()
            return compare(binary, baseline=action == Action.BASELINE)
        case _:
            assert_never(action)


if __name__ == "__main__":
    raise SystemExit(main())
