# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# uv run scripts/check-base-constants.py self-check
from __future__ import annotations

from collections import Counter
from typing import Final

from scripts.constructor_reference import ROOT, Case, Lane, decode_hex
from scripts.constructor_reference import self_check as constructor_self_check
from scripts.r_value_literal import literal
from scripts.r_value_literal import self_check as literal_self_check
from scripts.stop_capture import comparator_self_check, digest
from scripts.stop_reference import PROFILE

FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/base-constants-r-4.6.0.tsv"
FIXTURE_SHA: Final = "65383f4996f6de3d81412f4bad0b06b3644ac9dce4454792d7d43946cb370818"


def read_cases() -> tuple[Case, ...]:
    assert digest(FIXTURE) == FIXTURE_SHA
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == PROFILE
    rows: list[Case] = []
    for expected_id, line in enumerate(lines[1:], 1):
        identifier, source, storage, fingerprint, error, warnings, stdout = line.split("\t")
        assert identifier == str(expected_id)
        assert source and warnings == stdout == "-"
        row = Case(expected_id, Lane.REPRESENTED, source, storage, fingerprint, decode_hex(error), (), "")
        assert bool(row.error) == (storage == "error")
        if row.error:
            assert fingerprint == "-"
            name = source.removesuffix("(1L)")
            assert name in ("pi", "letters", "LETTERS", "month.name", "month.abb")
            assert row.error == f'could not find function "{name}"'
        else:
            assert fingerprint.split("|", 1)[0] == storage
            _ = literal(fingerprint)
        rows.append(row)
    assert len(rows) == 78
    assert Counter(row.storage for row in rows) == {
        "double": 8,
        "character": 33,
        "integer": 22,
        "NULL": 5,
        "logical": 5,
        "error": 5,
    }
    return tuple(rows)


def self_check() -> None:
    literal_self_check()
    constructor_self_check()
    comparator_self_check()
