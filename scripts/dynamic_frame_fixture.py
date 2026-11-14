# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Replay the immutable public D1 reference with the frozen GNU R runtime.

Run: python3 scripts/dynamic_frame_fixture.py verify
GNU R/Rscript must be on PATH and match the four published runtime digests.
NEW_OUTPUT may select a new evidence directory; existing directories are refused.
No ignored oracle data, primary source tree, or native executable is required.
"""
from __future__ import annotations

import os
from pathlib import Path
import sys
import time

from dynamic_frame_capture import Runtime, capture, discover, invocation, runtime_hashes, write_json
from dynamic_frame_records import MODES, Row, Verification, equal, sha, unhex
from dynamic_frame_reference import PINS, ROOT, read_reference, structure


def verify_row(row: Row, runtime: Runtime, output: Path) -> None:
    sources = output / 'sources'
    sources.mkdir(exist_ok=True)
    direct = sources / (row['id'] + '.R')
    typed = sources / (row['id'] + '.typed.R')
    observed = sources / (row['id'] + '.structural.R')
    _ = direct.write_bytes(row['source'])
    _ = typed.write_bytes(row['typed_source'])
    _ = observed.write_bytes(row['typed_source'] + (ROOT / 'scripts/dynamic_frame_observer.R').read_bytes())
    for mode in MODES:
        actual = capture(invocation(mode, typed if mode == 'typed' else direct, runtime),
                         runtime, output / 'fresh' / row['id'] / mode)
        for original in row['raw'].values():
            if row['contract'] == 'supported' or mode == 'typed':
                equal(original[mode], actual)
            else:
                assert actual['status'] == original[mode]['status'] == 0
                assert b'<bytecode: 0x' in unhex(actual['stdout'])
    for repeat in ('run-1', 'run-2'):
        actual = capture(invocation('structural', observed, runtime), runtime,
                         output / repeat / row['id'] / 'structural')
        stdout, marker, node = unhex(actual['stdout']).partition(b'DFNODE1\n')
        assert marker + node == row['node'], 'structural observation differs'
        if marker:
            _ = structure(marker + node)
        merged = unhex(actual['merged']).partition(b'DFNODE1\n')[0]
        original_typed = actual.copy()
        original_typed['stdout'] = stdout.hex()
        original_typed['merged'] = merged.hex()
        equal(original_typed, row['raw'][repeat]['typed'])


def public_inputs() -> dict[str, str]:
    paths = sorted((ROOT / 'scripts').glob('dynamic_frame_*.py'))
    paths.append(ROOT / 'scripts/dynamic_frame_observer.R')
    paths.extend(ROOT / 'crates/r_ust_eval/tests/fixtures' / name for name in PINS)
    return {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in paths}


def verify() -> None:
    inputs = public_inputs()
    originals = read_reference()
    supplemental = read_reference('dynamic-frame-source-r-4.6.0.tsv')
    rows = originals + supplemental
    assert len(originals) == 184 and len(supplemental) == 30
    profile = originals[0]['profile']
    assert all(row['profile'] == profile for row in rows)
    destination = os.environ.get('NEW_OUTPUT')
    output = Path(destination).resolve() if destination else ROOT / '.omo/evidence/dynamic-frame-fixture-task1-2026-11-14' / f'public-verify-{time.time_ns()}'
    output.mkdir(parents=True)
    write_json(output / 'input-manifest.json', inputs)
    write_json(output / 'profile.json', profile)
    runtime = discover(profile, output)
    categories: dict[str, int] = {}
    pending: list[str] = []
    for index, row in enumerate(rows):
        verify_row(row, runtime, output)
        if index < len(originals):
            category = structure(row['node'])[1] if row['node'] else 'terminal-error'
            categories[category] = categories.get(category, 0) + 1
        if row['contract'] == 'pending-bytecode':
            pending.append(row['id'])
        print(row['id'], flush=True)
    assert categories == dict(logical=57, integer=27, environment=12, closure=5, NULL=4,
                              list=2, pairlist=1, **{'terminal-error': 76})
    after = runtime_hashes(runtime.executables, runtime.environment)
    write_json(output / 'runtime-after.json', after)
    assert after == runtime.hashes, 'runtime changed during capture'
    assert public_inputs() == inputs, 'public inputs changed during capture'
    write_json(output / 'verification.json', Verification(result='PASS', original=len(originals), cli=180,
        sdk_reference_only=4, supplemental=len(supplemental), categories=categories, pending_bytecode=pending,
        raw_byte_comparison='Supported rows and all typed captures only; pending bytecode direct output retained without byte equality.',
        observations=len(rows) * 8, source_hashes_reverified=False))
    artifacts = {str(path.relative_to(output)): sha(path.read_bytes()) for path in sorted(output.rglob('*')) if path.is_file()}
    write_json(output / 'manifest.json', artifacts)
    print((output / 'verification.json').read_text(), end='')
    print('EVIDENCE_DIRECTORY=' + str(output))


if __name__ == '__main__':
    assert sys.argv[1:] == ['verify']
    verify()
