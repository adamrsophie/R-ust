#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run self-check, baseline BINARY NEW_OUTPUT, or compare BINARY NEW_OUTPUT.

All 74 frozen subjects are exercised in five CLI modes, with separate and merged
streams. Baseline retains failures; it never establishes or changes expectations.
The exact typed SDK outcome gate must be run separately by the integration owner.
"""
from __future__ import annotations

from pathlib import Path
import sys

from eval_runtime_runner import run
from eval_runtime_self_check import self_check


def main() -> int:
    usage = "self-check | baseline BINARY NEW_OUTPUT | compare BINARY NEW_OUTPUT"
    assert len(sys.argv) in (2, 4), usage
    action = sys.argv[1]
    if action == "self-check":
        assert len(sys.argv) == 2, usage
        self_check()
        return 0
    assert action in ("baseline", "compare") and len(sys.argv) == 4, usage
    return run(Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve(), action)


if __name__ == "__main__":
    raise SystemExit(main())
