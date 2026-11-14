from __future__ import annotations

from dataclasses import replace
from enum import StrEnum
from typing import Final

from scripts.constructor_reference import Case, stream_valid
from scripts.constructor_reference import matches as shared_matches
from scripts.stop_capture import Invocation, Run, encode_bytes

LIMIT_SOURCE: Final = (
    "{options(warn=2);tryCatch(sequence(1,from=c(3,4)),error=function(e) cat(conditionMessage(e)));"
    "options(warn=1);sequence(1,from=c(5,6))}"
)
LIMIT_ERROR: Final = 'could not find function "tryCatch"'


class Outcome(StrEnum):
    PASS = "PASS"
    FAIL = "FAIL"
    LIMIT = "LIMIT"


def is_limit_case(row: Case) -> bool:
    return row.identifier == 69 and row.source == LIMIT_SOURCE


def matches(row: Case, runs: tuple[Run, Run], *, reference: bool) -> bool:
    if not is_limit_case(row):
        return shared_matches(row, runs, reference=reference)
    separate, merged = runs
    return (
        separate.status == merged.status == 0
        and not row.error
        and separate.stdout == row.stdout + "TRUE\n"
        and not merged.stderr
        and stream_valid(separate.stderr, row, merged=False, reference=reference)
        and merged.stdout.startswith(row.stdout)
        and stream_valid(merged.stdout[len(row.stdout) :], replace(row, stdout=""), merged=True, reference=reference)
    )


def classify(row: Case, runs: tuple[Run, Run]) -> Outcome:
    return Outcome.PASS if matches(row, runs, reference=False) else Outcome.FAIL


def sample_run(stdout: str, stderr: str, status: int) -> Run:
    return Run(
        Invocation(("fixture-self-check",), b""),
        status,
        stdout,
        stderr,
        encode_bytes(stdout.encode()),
        encode_bytes(stderr.encode()),
    )


def self_check(row: Case, ordinary: Case) -> None:
    assert is_limit_case(row) and not is_limit_case(ordinary)
    assert row.storage == "integer" and row.fingerprint == "integer|1|5" and not row.error
    assert row.stdout and len(row.warnings) == 1
    for reference in (False, True):
        warnings = "".join("Warning: " + warning + "\n" for warning in row.warnings)
        separate = sample_run(row.stdout + "TRUE\n", warnings, 0)
        merged = sample_run(row.stdout + warnings + "TRUE\n", "", 0)
        assert matches(row, (separate, merged), reference=reference)
        assert classify(row, (separate, merged)) == Outcome.PASS
        for altered in (
            (replace(separate, status=1), merged),
            (separate, replace(merged, status=1)),
            (replace(separate, stdout="TRUE\n"), merged),
            (replace(separate, stderr=""), merged),
            (separate, replace(merged, stdout=warnings + row.stdout + "TRUE\n")),
            (separate, replace(merged, stdout=row.stdout + warnings + warnings + "TRUE\n")),
            (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
            (separate, replace(merged, stderr="extra\n")),
        ):
            assert not matches(row, altered, reference=reference)
    separate = sample_run("", LIMIT_ERROR + "\n", 1)
    merged = sample_run(LIMIT_ERROR + "\n", "", 1)
    assert classify(row, (separate, merged)) == Outcome.FAIL
    assert not matches(row, (separate, merged), reference=False)
    assert classify(replace(row, identifier=68), (separate, merged)) == Outcome.FAIL
    assert classify(replace(row, source="sequence(1)"), (separate, merged)) == Outcome.FAIL
    for altered in (
        (replace(separate, status=0), merged),
        (separate, replace(merged, status=0)),
        (replace(separate, stdout="effect"), merged),
        (replace(separate, stderr=LIMIT_ERROR.replace("tryCatch", "sequence") + "\n"), merged),
        (replace(separate, stderr="Warning: extra\n" + separate.stderr), merged),
        (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
        (separate, replace(merged, stderr="extra\n")),
    ):
        assert classify(row, altered) == Outcome.FAIL
    passed = (sample_run("TRUE\n", "", 0), sample_run("TRUE\n", "", 0))
    assert classify(ordinary, passed) == Outcome.PASS
    assert classify(ordinary, (replace(passed[0], stdout="FALSE\n"), passed[1])) == Outcome.FAIL
