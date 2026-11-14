#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run (from repository root):
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# python3 scripts/check-environment-values.py self-check
# python3 scripts/check-environment-values.py baseline BINARY NEW_OUTPUT
# python3 scripts/check-environment-values.py compare BINARY NEW_OUTPUT
from __future__ import annotations

import json
import shutil
import sys
from dataclasses import asdict, dataclass
from enum import StrEnum
from pathlib import Path
from typing import assert_never

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from scripts.environment_values_reference import (
    FIXTURE, FIXTURE_SHA, PROFILE, ROOT, Reference, capture_pair, cli_rows,
    live_profile, native_matches, read_reference, reference_matches,
)
from scripts.environment_values_self_check import self_check
from scripts.stop_capture import MODES, Invocation, Mode, Run, digest, encode_bytes, invocation
from scripts.stop_report import reference_inputs


class Action(StrEnum):
    BASELINE = "baseline"
    COMPARE = "compare"
    SELF_CHECK = "self-check"


@dataclass(frozen=True, slots=True)
class Captured:
    id: str
    r: Run
    r_merged: Run
    passed: bool


@dataclass(frozen=True, slots=True)
class Comparison:
    id: str
    r: Run
    r_merged: Run
    native: Run
    native_merged: Run
    passed: bool


@dataclass(frozen=True, slots=True)
class Snapshot:
    output: Path
    reference: tuple[str, str]
    native: tuple[str, str]


def source_inputs() -> tuple[Path, ...]:
    return tuple(sorted((
        *ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"),
        *ROOT.glob("crates/**/Cargo.toml"), ROOT / "Cargo.toml", ROOT / "Cargo.lock",
        Path(__file__), ROOT / "scripts/environment_values_reference.py",
        ROOT / "scripts/environment_values_self_check.py", ROOT / "scripts/stop_capture.py",
        ROOT / "scripts/stop_report.py", ROOT / "scripts/stop_reference.py",
    )))


def check_mode(mode: Mode, rows: tuple[Reference, ...], snapshot: Snapshot) -> int:
    references: list[Captured] = []
    for row in rows:
        source = snapshot.output / "sources" / (row.identifier + ".R")
        pair = capture_pair(invocation(mode, snapshot.reference, source))
        references.append(Captured(row.identifier, *pair, reference_matches(row, *pair)))
    _ = (snapshot.output / (mode + "-reference.json")).write_text(json.dumps(
        {"mode": mode, "cases": [asdict(row) for row in references]}, default=encode_bytes, indent=2,
    ) + "\n")
    assert all(row.passed for row in references), f"{mode}: fresh R differs; raw captures retained"
    results: list[Comparison] = []
    for row, reference in zip(rows, references, strict=True):
        assert row.identifier == reference.id
        source = snapshot.output / "sources" / (row.identifier + ".R")
        actual = capture_pair(invocation(mode, snapshot.native, source))
        pair = reference.r, reference.r_merged
        results.append(Comparison(row.identifier, *pair, *actual, native_matches(row, pair, actual)))
    count = sum(row.passed for row in results)
    _ = (snapshot.output / (mode + ".json")).write_text(json.dumps(
        {"mode": mode, "comparisons": len(results), "passes": count, "cases": [asdict(row) for row in results]},
        default=encode_bytes, indent=2,
    ) + "\n")
    print(f"{mode}: {count}/{len(rows)} native; {len(references)}/{len(rows)} frozen R", flush=True)
    return count


def compare(binary: Path, output: Path, action: Action) -> int:
    """Retain every capture, snapshot and manifest even when native parity fails."""
    output.mkdir(parents=True, exist_ok=False)
    rows = read_reference()
    selected = cli_rows(rows)
    sources = source_inputs()
    binaries = binary, binary.parent / "Rscript"
    runtimes = reference_inputs()
    reference = str(runtimes[0]), str(runtimes[1])
    home, _ = capture_pair(Invocation((reference[0], "--vanilla", "--slave", "-e", "cat(R.home())"), b""))
    assert home.status == 0 and home.stderr == ""
    runtime_files = (*runtimes, Path(home.stdout) / "bin/exec/R", Path(home.stdout) / "lib/libR.dylib")
    inputs = (*sources, *binaries, *runtime_files)
    before = {str(path): digest(path) for path in inputs}
    _ = (output / "inputs-before.json").write_text(json.dumps(before, indent=2) + "\n")
    assert live_profile(reference) == (PROFILE, PROFILE)
    (output / "bin").mkdir()
    snapshots = output / "bin/r_ust_cli", output / "bin/Rscript"
    for src, dest in zip(binaries, snapshots, strict=True):
        _ = shutil.copy2(src, dest)
        assert digest(src) == digest(dest)
    (output / "sources").mkdir()
    for row in selected:
        _ = (output / "sources" / (row.identifier + ".R")).write_bytes(row.source)
    programs = tuple(sorted((output / "sources").iterdir()))
    program_hashes = {str(path): digest(path) for path in programs}
    snapshot_hashes = {str(path): digest(path) for path in snapshots}
    _ = (output / "programs-before.json").write_text(json.dumps(program_hashes, indent=2) + "\n")
    context = Snapshot(output, reference, (str(snapshots[0]), str(snapshots[1])))
    passes = sum(check_mode(mode, selected, context) for mode in MODES)
    after = {str(path): digest(path) for path in inputs}
    programs_after = {str(path): digest(path) for path in programs}
    snapshots_after = {str(path): digest(path) for path in snapshots}
    stable = before == after and sources == source_inputs()
    stable &= program_hashes == programs_after and snapshot_hashes == snapshots_after
    _ = (output / "verification.json").write_text(json.dumps({
        "action": action,
        "comparisons": len(selected) * len(MODES), "passes": passes, "reference_passes": 485,
        "fresh_reference_captures": 970, "native_captures": 970, "stable": stable,
        "fixture_sha256": FIXTURE_SHA, "profile": PROFILE,
        "accounting": [{"id": row.identifier, "surface": row.surface} for row in rows],
        "input_sha256_before": before, "input_sha256_after": after,
        "program_sha256_before": program_hashes, "program_sha256_after": programs_after,
        "snapshot_sha256_before": snapshot_hashes, "snapshot_sha256_after": snapshots_after,
        "binary_sha256": {str(path): before[str(path)] for path in binaries},
        "runtime_sha256": {str(path): before[str(path)] for path in runtime_files},
        "limits": [
            "Source hashes describe the current checkout; compilation binding to the supplied binary is not established.",
            "R launcher, executable and libR hashes are recorded; transitive runtime libraries are not bound.",
            "Only exact hexadecimal environment pointer tokens and documented CLI condition frames differ.",
            "28 SDK and 3 reference-only rows remain accounted; identity, lifetime and GC are separate contracts.",
        ],
    }, indent=2) + "\n")
    assert stable, "inputs changed during capture"
    assert digest(FIXTURE) == FIXTURE_SHA
    return passes


def main() -> int:
    if sys.flags.optimize:
        raise SystemExit("verification requires assertions (do not use -O)")
    assert len(sys.argv) in (2, 4), "Usage: check-environment-values.py self-check | [baseline|compare] BINARY NEW_OUTPUT"
    action = Action(sys.argv[1])
    match action:
        case Action.SELF_CHECK:
            assert len(sys.argv) == 2
            self_check()
            return 0
        case Action.BASELINE | Action.COMPARE:
            assert len(sys.argv) == 4
            passes = compare(Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve(), action)
            return int(action == Action.COMPARE and passes != 485)
        case _:
            assert_never(action)


if __name__ == "__main__":
    raise SystemExit(main())
