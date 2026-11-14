#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run:
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# python3 scripts/check-environment-values.py reference NEW_OUTPUT
from __future__ import annotations

import csv
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from scripts.environment_values_reference import (
    FIXTURE, FIXTURE_SHA, PROFILE, ROOT, Reference, capture, capture_pair,
    cli_rows, live_profile, normalized, read_reference, reference_matches,
)
from scripts.stop_capture import MODES, Invocation, Run, digest, encode_bytes, invocation
from scripts.stop_report import reference_inputs

CASES: Final = ROOT / "scripts/environment-values-cases.tsv"
CASES_SHA: Final = "85f97aecfb76fc0bb2ee57c2aa8c4746b4879211df8e9d1aec7d348d4171c341"
ORACLE: Final = ROOT / "scripts/environment-values-oracle.R"
PRELUDE: Final = 'options(warn=1L, warnPartialMatchArgs=FALSE, keep.source=FALSE, digits=7L, scipen=0L, width=80L, deparse.cutoff=60L)'


@dataclass(frozen=True, slots=True)
class Observation:
    reference: Reference
    fields: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Capture:
    id: str
    direct: Run
    merged: Run
    typed: Run
    passed: bool


def observations() -> tuple[Observation, ...]:
    """Bind typed expectations and original observation boundaries to frozen source bytes."""
    rows = read_reference()
    assert digest(CASES) == CASES_SHA, "immutable observation inputs changed"
    with FIXTURE.open(newline="") as stream:
        gold = list(csv.reader((line for line in stream if not line.startswith("#")), delimiter="\t"))
    with CASES.open(newline="") as stream:
        cases = list(csv.reader(stream, delimiter="\t"))
    assert cases[0] == ["id", "category", "setup_hex", "subject_hex", "after_hex", "expected_status"]
    result: list[Observation] = []
    for row, fields, case in zip(rows, gold[1:], cases[1:], strict=True):
        assert len(case) == 6 and case[0] == row.identifier and case[1] == fields[1]
        assert int(case[5]) == row.status
        source = b"\n".join((PRELUDE.encode(), *(bytes.fromhex(value) for value in case[2:5]),
                               b'cat("AFTER_SUBJECT_SENTINEL\\n")', b""))
        assert source == row.source, (row.identifier, "observation source differs")
        result.append(Observation(row, tuple(fields[:12])))
    return tuple(result)


def typed_fields(text: str) -> tuple[str, ...]:
    """Decode only the three printable hex fields before applying the pointer exclusion."""
    assert text.endswith("\n") and text.count("\n") == 1, "invalid typed record framing"
    fields = text[:-1].split("\t")
    assert len(fields) == 13, "invalid typed record columns"
    return tuple(normalized(bytes.fromhex(value).decode()) if 9 <= index <= 11 else value
                 for index, value in enumerate(fields))


def typed_matches(row: Observation, run: Run) -> bool:
    expected = "\t".join((*row.fields, (row.reference.error or "").encode().hex())) + "\n"
    return run.status == 0 and run.stderr == "" and typed_fields(run.stdout) == typed_fields(expected)


def reference_capture(output: Path, sources: tuple[Path, ...]) -> None:
    """Capture fresh R observations twice; never write or derive the expected fixture."""
    output.mkdir(parents=True, exist_ok=False)
    rows = observations()
    runtimes = reference_inputs()
    executables = str(runtimes[0]), str(runtimes[1])
    home = capture(Invocation((executables[0], "--vanilla", "--slave", "-e", "cat(R.home())"), b""), False)
    _ = (output / "runtime-home.json").write_text(json.dumps(asdict(home), default=encode_bytes, indent=2) + "\n")
    assert home.status == 0 and home.stderr == ""
    runtime_files = (*runtimes, Path(home.stdout) / "bin/exec/R", Path(home.stdout) / "lib/libR.dylib")
    inputs = (*sources, *runtime_files)
    before = {str(path): digest(path) for path in inputs}
    _ = (output / "inputs-before.json").write_text(json.dumps(before, indent=2) + "\n")
    (output / "sources").mkdir()
    for row in rows:
        _ = (output / "sources" / (row.reference.identifier + ".R")).write_bytes(row.reference.source)
    programs = tuple(sorted((output / "sources").iterdir()))
    program_hashes = {str(path): digest(path) for path in programs}
    _ = (output / "programs-before.json").write_text(json.dumps(program_hashes, indent=2) + "\n")
    captures: list[Path] = []
    capture_hashes: dict[str, str] = {}
    successful = False
    try:
        profile = live_profile(executables)
        _ = (output / "profile.json").write_text(json.dumps(profile) + "\n")
        iterations: list[tuple[Capture, ...]] = []
        for iteration in (1, 2):
            current: list[Capture] = []
            for row in rows:
                identifier = row.reference.identifier
                source = output / "sources" / (identifier + ".R")
                direct, merged = capture_pair(invocation("file", executables, source))
                typed = capture(Invocation((executables[0], "--vanilla", "--slave", "-f", str(ORACLE),
                                            "--args", str(CASES), identifier), b""), False)
                raw = output / f"run{iteration}-{identifier}.json"
                _ = raw.write_text(json.dumps({"id": identifier, "direct": asdict(direct),
                    "merged": asdict(merged), "typed": asdict(typed)}, default=encode_bytes, indent=2) + "\n")
                captures.append(raw)
                capture_hashes[str(raw)] = digest(raw)
                passed = reference_matches(row.reference, direct, merged) and typed_matches(row, typed)
                current.append(Capture(identifier, direct, merged, typed, passed))
            iterations.append(tuple(current))
            print(f"reference run {iteration}: {sum(row.passed for row in current)}/128", flush=True)
        repeatable = all(
            first.id == second.id and first.passed and second.passed
            and typed_fields(first.typed.stdout) == typed_fields(second.typed.stdout)
            and normalized(first.direct.stdout) == normalized(second.direct.stdout)
            and normalized(first.direct.stderr) == normalized(second.direct.stderr)
            and normalized(first.merged.stdout) == normalized(second.merged.stdout)
            for first, second in zip(iterations[0], iterations[1], strict=True)
        )
        mode_passes: list[int] = []
        for mode in MODES:
            passes = 0
            for row in cli_rows(tuple(row.reference for row in rows)):
                pair = capture_pair(invocation(mode, executables, output / "sources" / (row.identifier + ".R")))
                raw = output / f"{mode}-{row.identifier}.json"
                _ = raw.write_text(json.dumps({"id": row.identifier, "mode": mode,
                    "direct": asdict(pair[0]), "merged": asdict(pair[1])}, default=encode_bytes, indent=2) + "\n")
                captures.append(raw)
                capture_hashes[str(raw)] = digest(raw)
                passes += reference_matches(row, *pair)
            mode_passes.append(passes)
            print(f"reference {mode}: {passes}/97", flush=True)
        _ = (output / "captures-before.json").write_text(json.dumps(capture_hashes, indent=2) + "\n")
        _ = (output / "verification.json").write_text(json.dumps({
            "fixture_sha256": FIXTURE_SHA, "profile": PROFILE,
            "accounting": [{"id": row.reference.identifier, "surface": row.reference.surface} for row in rows],
            "typed_passes": [sum(row.passed for row in run) for run in iterations],
            "repeatable": repeatable, "modes": MODES, "mode_passes": mode_passes,
            "direct_runs": 512, "typed_runs": 256, "five_mode_runs": 970,
            "limits": ["R-only observations; no native SDK, lifetime or GC pass claim.",
                       "Only exact environment pointer tokens are excluded.",
                       "Runtime launchers, executable and libR are bound; transitive libraries are not."],
        }, indent=2) + "\n")
        assert repeatable and mode_passes == [97] * len(MODES), "R differs; raw captures retained"
        assert capture_hashes == {str(path): digest(path) for path in captures}, "capture bytes changed"
        successful = True
    finally:
        after = {str(path): digest(path) for path in inputs}
        programs_after = {str(path): digest(path) for path in programs}
        captures_after = {str(path): digest(path) for path in captures}
        _ = (output / "integrity.json").write_text(json.dumps({
            "successful": successful, "inputs_before": before, "inputs_after": after,
            "programs_before": program_hashes, "programs_after": programs_after,
            "captures_before": capture_hashes, "captures_after": captures_after,
        }, indent=2) + "\n")
        assert before == after and program_hashes == programs_after, "inputs changed during capture"
        assert capture_hashes == captures_after, "capture bytes changed"
