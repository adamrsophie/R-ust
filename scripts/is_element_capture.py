from __future__ import annotations

from dataclasses import dataclass, replace
from enum import StrEnum
from typing import Final

from scripts.constructor_reference import Case, matches
from scripts.stop_capture import Invocation, Run, encode_bytes


class Outcome(StrEnum):
    PASS = "PASS"
    FAIL = "FAIL"
    LIMIT = "LIMIT"


@dataclass(frozen=True, slots=True)
class Limitation:
    identifier: int
    source: str
    error: str


LIMITS: Final = (
    Limitation(14, "is.element(list(1L,'a',NULL),list(1L,NULL))", "cannot match value of type List"),
    Limitation(15, "is.element(list(1L,2L),1L)", "cannot match value of type List"),
    Limitation(40, "is.element(structure(c(1L,2L),class='custom'),2L)", 'could not find function "structure"'),
)


def limitation(row: Case) -> Limitation | None:
    return next((item for item in LIMITS if (row.identifier, row.source) == (item.identifier, item.source)), None)


def classify(row: Case, runs: tuple[Run, Run]) -> Outcome:
    boundary = limitation(row)
    if boundary is not None:
        diagnostic = replace(row, storage="error", fingerprint="-", error=boundary.error, warnings=(), stdout="")
        return Outcome.LIMIT if matches(diagnostic, runs, reference=False) else Outcome.FAIL
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


def self_check(rows: tuple[Case, ...]) -> None:
    for row in rows:
        boundary = limitation(row)
        if boundary is None:
            continue
        separate = sample_run("", boundary.error + "\n", 1)
        merged = sample_run(boundary.error + "\n", "", 1)
        assert classify(row, (separate, merged)) == Outcome.LIMIT
        assert classify(replace(row, identifier=27), (separate, merged)) == Outcome.FAIL
        assert classify(replace(row, source="is.element(1L,1L)"), (separate, merged)) == Outcome.FAIL
        for altered in (
            (replace(separate, status=0), merged),
            (separate, replace(merged, status=0)),
            (replace(separate, stdout="effect"), merged),
            (replace(separate, stderr="another failure\n"), merged),
            (replace(separate, stderr="Warning: extra\n" + separate.stderr), merged),
            (separate, replace(merged, stdout=merged.stdout + "AFTER\n")),
            (separate, replace(merged, stderr="extra\n")),
            (sample_run("TRUE\n", "", 0), sample_run("TRUE\n", "", 0)),
        ):
            assert classify(row, altered) == Outcome.FAIL
    row = rows[26]
    assert row.identifier == 27 and limitation(row) is None
    assert row.stdout == "1" and row.fingerprint == "logical|1|T"
    passed = (sample_run("1TRUE\n", "", 0), sample_run("1TRUE\n", "", 0))
    failed = (sample_run("0FALSE\n", "", 0), sample_run("0FALSE\n", "", 0))
    assert classify(row, passed) == Outcome.PASS
    assert classify(row, failed) == Outcome.FAIL
    assert classify(row, (replace(passed[0], stdout="TRUE\n"), passed[1])) == Outcome.FAIL
