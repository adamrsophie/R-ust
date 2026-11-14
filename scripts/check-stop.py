#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# python3 scripts/check-stop.py reference
# python3 scripts/check-stop.py baseline [target/debug/r_ust_cli]
# python3 scripts/check-stop.py compare [target/debug/r_ust_cli]
# python3 scripts/check-stop.py report-check
from __future__ import annotations

import json
import shutil
import sys
import tempfile
from dataclasses import asdict
from enum import StrEnum
from pathlib import Path
from typing import assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.stop_capture import (
    FIXTURE_SHA,
    MODES,
    Comparison,
    comparator_self_check,
    digest,
    encode_bytes,
    invocation,
    matches,
    run,
)
from scripts.stop_reference import (
    ARTIFACTS,
    FIXTURE,
    ROOT,
    generate_reference,
    live_profile,
    read_reference,
)
from scripts.stop_report import reference_inputs, report_check, source_inputs


class Action(StrEnum):
    REFERENCE = "reference"
    BASELINE = "baseline"
    COMPARE = "compare"
    REPORT_CHECK = "report-check"


def compare(binary: Path, *, baseline: bool) -> int:
    comparator_self_check()
    profiles = live_profile()
    rows = read_reference()
    assert digest(FIXTURE) == FIXTURE_SHA
    assert len(rows) == 121 and sum(row.spec.lane == "limited-condition" for row in rows) == 1
    assert all(not row.warnings for row in rows), "warning cases need ordered stream comparison"
    inputs = source_inputs()
    hashes = {str(path): digest(path) for path in inputs}
    original = (binary, binary.parent / "Rscript")
    binary_hashes = {str(path): digest(path) for path in original}
    references = {str(path): digest(path) for path in reference_inputs()}
    comparisons: list[Comparison] = []
    directory = ARTIFACTS / ("baseline" if baseline else "compare")
    directory.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="r-ust-stop-") as temporary:
        snapshots = tuple(Path(temporary) / name for name in ("r_ust_cli", "Rscript"))
        for src, dest in zip(original, snapshots, strict=True):
            _ = shutil.copy2(src, dest)
        assert all(digest(src) == digest(dest) for src, dest in zip(original, snapshots, strict=True))
        for mode in MODES:
            for row in rows:
                source = Path(temporary) / "stop source.R"
                program = row.spec.source
                if row.kind != "error":
                    program = (
                        "answer<-{"
                        + program
                        + '};cat(typeof(answer),":",paste(as.character(answer),collapse="|"),"\\n",sep="")'
                    )
                else:
                    program += ';cat("AFTER\\n")'
                _ = source.write_text("options(warn=1);" + program + "\n")
                r = run(invocation(mode, ("R", "Rscript"), source))
                r_merged = run(invocation(mode, ("R", "Rscript"), source), merged=True)
                native = run(invocation(mode, (str(snapshots[0]), str(snapshots[1])), source))
                native_merged = run(invocation(mode, (str(snapshots[0]), str(snapshots[1])), source), merged=True)
                reference_valid = matches(row, r, r_merged, reference=True)
                assert reference_valid, f"live R disagrees with fixture: {mode}/{row.spec.label}: {r}"
                supported = True
                passed = supported and matches(row, native, native_merged, reference=False)
                comparisons.append(
                    Comparison(
                        mode,
                        row,
                        r,
                        native,
                        r_merged,
                        native_merged,
                        supported,
                        passed,
                        (r.stdout_base64, r.stderr_base64) == (native.stdout_base64, native.stderr_base64),
                        r_merged.stdout_base64 == native_merged.stdout_base64,
                    )
                )
            print(f"Checked {mode}", flush=True)
    stable = all(digest(Path(path)) == value for path, value in (hashes | binary_hashes | references).items())
    stable &= inputs == source_inputs()
    failures = [row for row in comparisons if row.supported and not row.passed]
    report = directory / "report.json"
    _ = report.write_text(
        json.dumps(
            {
                "fixture_sha256": FIXTURE_SHA,
                "source_sha256": hashes,
                "binary_sha256": binary_hashes,
                "reference_sha256": references,
                "reference_profile": profiles,
                "stable": stable,
                "baseline": baseline,
                "cases": [asdict(row) for row in comparisons],
                "limits": [
                    "The original condition-object row is required to match its frozen custom error payload and termination across all five modes.",
                    "Requested messages, output, exit and event order are compared; call headers, Calls and Execution halted differ.",
                    "Raw call metadata is retained; call objects/tracebacks/translations/byte encodings are not certified.",
                ],
            },
            default=encode_bytes,
            indent=2,
        )
        + "\n"
    )
    summary = ["report\t" + str(report) + "\t" + digest(report)]
    summary.extend(f"input\t{path}\t{value}" for path, value in (hashes | binary_hashes | references).items())
    summary.extend(
        f"row\t{row.mode}/{row.reference.identifier}/{row.reference.spec.label}\t"
        + ("PASS" if row.passed else "FAIL" if row.supported else "LIMITED")
        for row in comparisons
    )
    _ = (directory / "summary.tsv").write_text("\n".join(summary) + "\n")
    for row in failures:
        print(f"FAIL {row.mode}/{row.reference.spec.label}")
    print(
        f"{sum(row.passed for row in comparisons)}/605 supported comparisons; {len(failures)} failures; 0 LIMITED; stable={stable}"
    )
    return 0 if baseline else int(bool(failures) or not stable)


def main() -> int:
    try:
        mode = Action(sys.argv[1] if len(sys.argv) > 1 else "compare")
    except ValueError:
        raise SystemExit("Usage: check-stop.py [reference|baseline|compare|report-check] [binary]") from None
    match mode:
        case Action.REFERENCE:
            generate_reference()
            assert digest(FIXTURE) == FIXTURE_SHA
            print(f"{len(read_reference())} reference rows: {FIXTURE_SHA}")
            return 0
        case Action.BASELINE | Action.COMPARE:
            binary = Path(sys.argv[2] if len(sys.argv) > 2 else ROOT / "target/debug/r_ust_cli").resolve()
            return compare(binary, baseline=mode == Action.BASELINE)
        case Action.REPORT_CHECK:
            return report_check()
        case _:
            assert_never(mode)


if __name__ == "__main__":
    raise SystemExit(main())
