#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run:
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# python3 scripts/check-environment-values.py self-check
from __future__ import annotations

import json
from dataclasses import asdict, replace

from scripts.environment_values_reference import (
    POINTER, Reference, cli_rows, native_matches, read_reference, reference_matches,
)
from scripts.stop_capture import Invocation, Run, encode_bytes
from scripts.stop_report import json_decoder, read_run


def streams(run: Run, stdout: str, stderr: str) -> Run:
    return replace(
        run, stdout=stdout, stderr=stderr,
        stdout_base64=encode_bytes(stdout.encode()), stderr_base64=encode_bytes(stderr.encode()),
    )


def synthetic(row: Reference, reference: tuple[Run, Run]) -> tuple[Run, Run]:
    separate, merged = reference
    if row.error is not None:
        error = "type error: " + row.error + "\n"
        return streams(separate, separate.stdout, error), streams(merged, separate.stdout + error, "")
    output = merged.stdout
    if separate.stderr:
        assert output.count(separate.stderr) == 1
        output = output.replace(separate.stderr, row.warning, 1)
    return streams(separate, separate.stdout, row.warning), streams(merged, output, "")


def self_check() -> None:
    """Guard comparison permissiveness with independent frozen R records, not native success claims."""
    checks: list[tuple[str, str]] = []
    for row in cli_rows(read_reference()):
        blank = Run(Invocation(("synthetic-self-check",), row.source), row.status, "", "", "", "")
        reference = streams(blank, row.streams.stdout, row.streams.stderr), streams(blank, row.streams.merged, "")
        separate, merged = synthetic(row, reference)
        assert native_matches(row, reference, (separate, merged))
        checks.append((row.identifier, "accepted-permitted-CLI-framing"))
        for name, mutated in (
            ("separate-status", (replace(separate, status=separate.status + 1), merged)),
            ("merged-status", (separate, replace(merged, status=merged.status + 1))),
            ("stdout", (streams(separate, "X" + separate.stdout, separate.stderr), merged)),
            ("stderr", (streams(separate, separate.stdout, separate.stderr + "X"), merged)),
            ("merged-stdout", (separate, streams(merged, "X" + merged.stdout, merged.stderr))),
            ("merged-stderr", (separate, streams(merged, merged.stdout, "X"))),
        ):
            assert not native_matches(row, reference, mutated), (row.identifier, name)
            checks.append((row.identifier, name))
        gold, gold_merged = reference
        for name, mutated in (
            ("R-separate-status", (replace(gold, status=gold.status + 1), gold_merged)),
            ("R-merged-status", (gold, replace(gold_merged, status=gold_merged.status + 1))),
            ("R-stdout", (streams(gold, "X" + gold.stdout, gold.stderr), gold_merged)),
            ("R-stderr", (streams(gold, gold.stdout, gold.stderr + "X"), gold_merged)),
            ("R-merged-stdout", (gold, streams(gold_merged, "X" + gold_merged.stdout, ""))),
            ("R-merged-stderr", (gold, streams(gold_merged, gold_merged.stdout, "X"))),
        ):
            assert not reference_matches(row, *mutated), (row.identifier, name)
            checks.append((row.identifier, name))
            try:
                _ = native_matches(row, mutated, (separate, merged))
            except AssertionError:
                checks.append((row.identifier, name + "-native-guard"))
            else:
                raise AssertionError((row.identifier, name, "invalid R reference was trusted"))
        assert read_run(json_decoder()(json.dumps(asdict(separate), default=encode_bytes))) == separate
        checks.append((row.identifier, "serialized-capture-roundtrip"))
        if row.error is not None:
            wrong = replace(row, error=row.error + "X")
            try:
                _ = native_matches(wrong, reference, synthetic(wrong, reference))
            except AssertionError:
                checks.append((row.identifier, "independent-R-payload-tamper-rejected"))
            else:
                raise AssertionError((row.identifier, "R diagnostic payload was trusted"))
        if POINTER.search(separate.stdout):
            changed = (
                streams(separate, POINTER.sub("0x1234", separate.stdout), separate.stderr),
                streams(merged, POINTER.sub("0xABC", merged.stdout), merged.stderr),
            )
            assert native_matches(row, reference, changed)
            checks.append((row.identifier, "pointer-only-accepted"))
        if 'attr(,"name")' in separate.stdout:
            for name, old, new in (
                ("attribute-key", 'attr(,"name")', 'attr(,"wrong")'),
                ("attribute-value", '"Autoloads"', '"WRONG"'),
                ("attribute-loss", '\nattr(,"name")\n[1] "Autoloads"', ""),
                ("indent", '<environment:', ' <environment:'),
                ("fake-named-marker", '<environment: 0x', '<environment: Autoloads0x'),
            ):
                changed = (
                    streams(separate, separate.stdout.replace(old, new, 1), separate.stderr),
                    streams(merged, merged.stdout.replace(old, new, 1), merged.stderr),
                )
                assert not native_matches(row, reference, changed), (row.identifier, name)
                checks.append((row.identifier, name))
    print(json.dumps({
        "comparator_checks": len(checks), "checks": checks,
        "scope": "Synthetic comparator inputs from frozen independent R records; no native environment-pass claim.",
    }, indent=2))
