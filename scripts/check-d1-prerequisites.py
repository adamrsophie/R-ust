#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run capture NEW_DIR, compare BINARY NEW_DIR, or self-check."""
from pathlib import Path
import sys

from d1_prerequisite_runner import run


def main() -> int:
    usage = 'capture NEW_DIR | compare BINARY NEW_DIR | self-check'
    if sys.argv[1:] == ['self-check']:
        from d1_prerequisite_self_check import self_check
        self_check()
        return 0
    assert len(sys.argv) >= 2, usage
    if sys.argv[1] == 'capture':
        assert len(sys.argv) == 3, usage
        return int(not run(Path(sys.argv[2]).resolve(), None))
    assert len(sys.argv) == 4 and sys.argv[1] == 'compare', usage
    return int(not run(Path(sys.argv[3]).resolve(), Path(sys.argv[2]).resolve()))


if __name__ == '__main__':
    raise SystemExit(main())
