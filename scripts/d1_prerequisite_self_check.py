"""Mutation checks for frozen program, typed result, and output contracts."""
from collections.abc import Callable
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from d1_prerequisite_cases import CASES
from d1_prerequisite_runner import FIXTURE, exact, read_fixture
from dynamic_frame_records import Capture, Invocation


def rejected(action: Callable[[], bool]) -> None:
    try:
        accepted = action()
    except AssertionError:
        return
    assert not accepted, 'mutated input accepted'


def validate_fixture(data: bytes) -> bool:
    with tempfile.TemporaryDirectory(prefix='d1-prerequisites-mutant-') as temporary:
        path = Path(temporary) / FIXTURE.name
        _ = path.write_bytes(data)
        return bool(read_fixture(path))


def self_check() -> None:
    rows = read_fixture()
    original = FIXTURE.read_bytes()
    rejected_labels: list[str] = []
    for label, column, replacement in (
        ('class', 3, b'condition'), ('visibility', 3, b'invisible'),
        ('missingness', 3, b'missing'), ('id', 0, b'changed'),
        ('source', 2, b'changed')):
        fields = rows[0].split('\t')
        fields[column] = replacement.decode() if column == 0 else replacement.hex()
        changed_fixture = original.replace(rows[0].encode(), '\t'.join(fields).encode(), 1)
        rejected(lambda: validate_fixture(changed_fixture))
        rejected_labels.append(label)
    rejected(lambda: validate_fixture(original + rows[0].encode() + b'\n'))
    rejected(lambda: validate_fixture(original.replace(rows[0].encode() + b'\n', b'', 1)))
    rejected_labels.extend(('duplicate-case', 'missing-case'))
    baseline = Capture(stdout=b'CALLEXIT'.hex(), stderr=b'error\n'.hex(), merged=b'CALLEXITerror\n'.hex(),
                       status=1, invocation=Invocation(argv=['R', '-e', 'literal'], stdin=None))
    for label in ('effect-order', 'diagnostic', 'status'):
        changed = baseline.copy()
        if label == 'effect-order':
            changed['merged'] = b'EXITCALLerror\n'.hex()
        elif label == 'diagnostic':
            changed['stderr'] = b'changed\n'.hex()
        else:
            changed['status'] = 0
        rejected(lambda: exact(baseline, changed))
        rejected_labels.append(label)
    with tempfile.TemporaryDirectory(prefix='d1-prerequisites-self-check-') as temporary:
        command = [sys.executable, '-O', str(Path(__file__).with_name('check-d1-prerequisites.py')), 'self-check']
        result = subprocess.run(command, capture_output=True, check=False, timeout=30)
        _ = (Path(temporary) / 'stderr').write_bytes(result.stderr)
        assert result.returncode != 0 and b'non-optimized Python' in result.stderr
        rejected_labels.append('optimized-python')
    assert len(rows) == len(CASES) == 33
    print(json.dumps(dict(result='PASS', cases=len(rows), rejected=rejected_labels), sort_keys=True))
