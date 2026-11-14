# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 scripts/dynamic_frame_reference.py self-check
"""Strict public D1 reference reader; no native execution or dput parsing.

DFNODE1 is followed by visibility and one recursive node. Each node header is
TAB-separated tag, length, names, payload, child count, then children and `end`.
Names are `-` (absent) or ordered comma-separated UTF-8 hex atoms; `_` is empty.
Logical/integer payloads are decimal, doubles exact hexadecimal R %a, and strings
hex atoms; NA is explicit. Closure children are formals, body, and environment.
Environment payloads are four named-role predicate results in fixed order:
globalenv(), baseenv(), .BaseNamespaceEnv, emptyenv(). All FALSE means other;
it does not assert pointer identity. Missing arguments have their own node.
Raw fields preserve both original runs in all six modes, including typed dput.
The direct source and typed setup/subject are independently pinned; a supplemental
multi-expression program retains exact direct argv bytes and a final typed subject.
"""
from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
from typing import Final

from dynamic_frame_records import MODES, Capture, Invocation, Pin, Row, equal, parse_profile, parse_raw, sha, unhex
from dynamic_frame_records import normalized as normalized
from dynamic_frame_nodes import structure as structure

ROOT: Final = Path(__file__).resolve().parents[1]
FIXTURES: Final = ROOT / 'crates/r_ust_eval/tests/fixtures'
PINS: Final[dict[str, Pin]] = {'dynamic-frame-source-r-4.6.0.tsv': {'fixture': 'ff84d5ed63ed6527bc6d00fb017f36f7335aae5f1c8f846159b7d18a2b602ae0', 'count': 30, 'source': 'b0e22328faf6eb74909d4e5c97c06afbf2b3e56f7320925b1d5d0fe395d470e2', 'typed_source': '3dfe0d0b2762a133c02f513d76c255fef4be98e692a130d5bb2e30715f765185', 'raw': '5d9ab3fcec444d5b24c79a4757c34a309ff48bc7cfd2e5f30c34a191f70f61ac'}, 'dynamic-frames-r-4.6.0.tsv': {'fixture': 'ce2cea3876495f954d15ad00f04e126f246c5dd1c86b603808aa8105e51d8923', 'count': 184, 'source': 'e7d7c98a91ee279196117914b0a5d84d3ccdcdbc36ee21e35459f8fc87379fdd', 'typed_source': '7f563e2a59c468d2f151910870312071fe93f55e7bcd4e9768aa9fd007b0900c', 'raw': '6bd07eb40ab9b4388413b3560adf1fa659c26b6fdbb7be9e7ecddc2367bc0224'}}
HEADER: Final = 'id\tscope\tsetup_hex\texpression_hex\tsource_hex\ttyped_source_hex\traw_hex\tnode_hex\tprofile_hex\tcontract'
def parse(data: bytes, count: int) -> tuple[Row, ...]:
    assert data.endswith(b'\n')
    lines = data.decode().splitlines()
    assert lines[0] == HEADER
    rows: list[Row] = []
    for line in lines[1:]:
        fields = line.split('\t')
        assert len(fields) == 10
        identifier, scope = fields[:2]
        assert re.fullmatch(r'[a-z][a-z0-9_]*', identifier)
        assert scope in ('cli', 'sdk-reference-only')
        setup, expression, source, typed, raw, tree, profile = map(unhex, fields[2:9])
        contract = fields[9]
        assert contract in ('supported', 'pending-bytecode')
        assert contract != 'pending-bytecode' or scope == 'sdk-reference-only'
        assert expression and source
        assert typed == (setup + b'\n' if setup else b'') + b'oracle_result <- .Internal(withVisible(' + expression + b"))\ncat('ORACLE_TYPE:',typeof(oracle_result$value),'\\n',sep='')\ndput(oracle_result)\n"
        records = parse_raw(raw)
        for run in records.values():
            for mode in MODES:
                capture = run[mode]
                if mode in ('expression', 'rscript-expression'):
                    assert capture['invocation']['argv'][-1].encode() == source
                if mode == 'stdin':
                    incoming = capture['invocation']['stdin']
                    assert incoming is not None and incoming.encode() == source
        if records['run-1']['typed']['status']:
            assert tree == b''
        else:
            visible, category = structure(tree)
            output = unhex(records['run-1']['typed']['stdout'])
            assert b'ORACLE_TYPE:' + category.encode() + b'\n' in output
            assert re.search(rb', visible = ' + (b'TRUE' if visible == 'visible' else b'FALSE') + rb'\)\n$', output)
        rows.append(Row(id=identifier, scope=scope, setup=setup, expression=expression, source=source,
                        typed_source=typed, raw=records, node=tree, profile=parse_profile(profile), contract=contract))
    assert len(rows) == len({row['id'] for row in rows}) == count
    return tuple(rows)


def read_reference(name: str = 'dynamic-frames-r-4.6.0.tsv', directory: Path = FIXTURES) -> tuple[Row, ...]:
    data = (directory / name).read_bytes()
    pin = PINS[name]
    assert sha(data) == pin['fixture'], 'immutable fixture changed'
    rows = parse(data, pin['count'])
    assert sha(json.dumps([row['source'].hex() for row in rows], sort_keys=True).encode()) == pin['source']
    assert sha(json.dumps([row['typed_source'].hex() for row in rows], sort_keys=True).encode()) == pin['typed_source']
    assert sha(json.dumps([row['raw'] for row in rows], sort_keys=True).encode()) == pin['raw']
    observer = sha((ROOT / 'scripts/dynamic_frame_observer.R').read_bytes())
    assert all(row['profile']['observer_sha256'] == observer for row in rows)
    if name == 'dynamic-frames-r-4.6.0.tsv':
        assert all(row['contract'] == 'supported' for row in rows)
        assert sum(row['scope'] == 'cli' for row in rows) == 180
    return rows


def rejected(action: Callable[[], tuple[Row, ...] | tuple[str, str] | None]) -> None:
    try:
        _ = action()
    except (AssertionError, ValueError, IndexError):
        return
    raise AssertionError('mutated input accepted')


def replace_field(lines: list[bytes], column: int, value: bytes) -> bytes:
    fields = lines[1].decode().rstrip('\n').split('\t')
    fields[column] = value.hex()
    return lines[0] + ('\t'.join(fields) + '\n').encode() + b''.join(lines[2:])


def loader_rejects(name: str, changed: bytes, scratch: Path) -> None:
    _ = (scratch / name).write_bytes(changed)
    try:
        _ = read_reference(name, scratch)
    except AssertionError as error:
        assert str(error) == 'immutable fixture changed', 'digest guard was bypassed'
        return
    raise AssertionError('production loader accepted corruption')


def optimized_probes(scratch: Path) -> dict[str, int]:
    reader = 'import sys; from pathlib import Path; import dynamic_frame_reference as r; r.read_reference(directory=Path(sys.argv[1]))'
    node = "import dynamic_frame_reference as r; r.structure(b'DFNODE1\\nvisible\\nunknown\\t0\\t-\\t_\\t0\\nend\\n')"
    results: dict[str, int] = {}
    for label, program in (('corrupt-fixture', reader), ('unknown-node', node)):
        for optimization in (False, True):
            command = [sys.executable, *(['-O'] if optimization else []), '-c', program, str(scratch)]
            result = subprocess.run(command, cwd=ROOT / 'scripts', capture_output=True, timeout=30, check=False)
            assert result.returncode != 0
            expected = b'non-optimized Python' if optimization else (
                b'immutable fixture changed' if label == 'corrupt-fixture' else b'unknown type')
            assert expected in result.stderr
            results[label + ('-optimized' if optimization else '-normal')] = result.returncode
    return results


def stream_checks() -> int:
    data = b'first\n<environment: 0xabc>\nwarning at 0xabc\nerror\n'
    original = Capture(stdout=data.hex(), stderr=b'Warning\nError\n'.hex(), merged=data.hex(), status=1,
                       invocation=Invocation(argv=['R'], stdin=None))
    pointer = original.copy()
    pointer['stdout'] = data.replace(b'<environment: 0xabc>', b'<environment: 0x123>').hex()
    equal(original, pointer)
    mutants: list[Capture] = []
    for value in (data[::-1], data + b'warning\n', data.replace(b'warning', b'changed'),
                  data.replace(b'at 0xabc', b'at 0x123'), data.replace(b'environment:', b'other:')):
        changed = original.copy()
        changed['stdout'] = value.hex()
        mutants.append(changed)
    for label in ('status', 'warning', 'order'):
        changed = original.copy()
        if label == 'status':
            changed['status'] = 0
        elif label == 'warning':
            changed['stderr'] += b'changed\n'.hex()
        else:
            changed['merged'] = data[::-1].hex()
        mutants.append(changed)
    for changed in mutants:
        rejected(lambda: equal(original, changed))
    return len(mutants)


def self_check() -> None:
    results: dict[str, str] = {}
    structural_mutations = 0
    with tempfile.TemporaryDirectory(prefix='dynamic-frame-reader-') as directory:
        scratch = Path(directory)
        for name, pin in PINS.items():
            rows = read_reference(name)
            data = (FIXTURES / name).read_bytes()
            lines = data.splitlines(keepends=True)
            mutations = {'duplicate': data + lines[1], 'missing': b''.join(lines[:-1]),
                         'hex': data.replace(b'\tcli\t', b'\tcli\tz', 1),
                         'scope': data.replace(b'\tcli\t', b'\tother\t', 1)}
            for label, changed in mutations.items():
                rejected(lambda: parse(changed, pin['count']))
                results[name + ':' + label] = 'schema-rejected'
            for column, label in ((4, 'source'), (5, 'typed-source'), (6, 'raw')):
                changed = replace_field(lines, column, unhex(lines[1].decode().split('\t')[column]) + b'changed')
                loader_rejects(name, changed, scratch)
                results[name + ':' + label] = 'loader-rejected'
            for label in ('status', 'warning', 'order'):
                raw = parse_raw(unhex(lines[1].decode().split('\t')[6]))
                capture = raw['run-1']['expression']
                if label == 'status':
                    capture['status'] = 1 - capture['status']
                elif label == 'warning':
                    capture['stderr'] += b'Warning: changed\n'.hex()
                else:
                    capture['merged'] = (b'changed order\n' + unhex(capture['merged'])).hex()
                loader_rejects(name, replace_field(lines, 6, json.dumps(raw, sort_keys=True).encode()), scratch)
                results[name + ':' + label] = 'loader-rejected'
            sample = next(row['node'] for row in rows if row['node'])
            header = sample.splitlines()[2].split(b'\t')[0]
            for changed in (sample.replace(b'\n' + header + b'\t', b'\nunknown\t', 1), sample + b'end\n',
                            sample.replace(b'\t1\t', b'\t999\t', 1), sample.replace(b'\t0\n', b'\t999\n', 1)):
                rejected(lambda: structure(changed))
                structural_mutations += 1
            _ = read_reference(name)
        subprocess_results = optimized_probes(scratch)
    print(json.dumps(dict(result='PASS', fixture_mutations=results, structural_mutations=structural_mutations,
                          stream_mutations=stream_checks(), subprocess_rejections=subprocess_results), sort_keys=True))


if __name__ == '__main__':
    assert sys.argv[1:] == ['self-check']
    self_check()
