"""Small attested prerequisite captures; typed native execution is a separate gate."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import json
from typing import TypedDict

from d1_prerequisite_cases import CASES, Case
from dynamic_frame_capture import Runtime, capture, discover, runtime_hashes, write_json
from dynamic_frame_compare import Conditions, Frame, native_matches
from dynamic_frame_records import Capture, Invocation, sha, unhex
from dynamic_frame_reference import ROOT, read_reference, structure

HEADER = 'id\tsetup_hex\texpression_hex\tnode_hex\tstatus\tconditions_hex\tframe'
FIXTURE = ROOT / 'crates/r_ust_eval/tests/fixtures/d1-prerequisites-r-4.6.0.tsv'
FIXTURE_SHA = '5415fd65cab4cb41a224a1886a44450e8ec009636ee98b881c0d1bc1620fac02'
class Result(TypedDict):
    id: str
    reference_two_runs: bool
    direct_cli: bool | str


class Report(TypedDict):
    reference_two_runs: bool
    count: int
    stable: bool
    direct_cli: bool | str
    native_typed: str
    fixture_sha256: str
    binary_sha256: str | None
    cases: list[Result]
    runtime_after: dict[str, str]


MARKER = re.compile(rb'^R_UST_D1_(WARNING|ERROR)=([0-9a-f]*)\n', re.MULTILINE)


@dataclass(frozen=True, slots=True)
class Observation:
    direct: Capture
    structural: Capture
    node: bytes
    conditions: tuple[tuple[str, str], ...]


def exact(left: Capture, right: Capture) -> bool:
    return all(left[key] == right[key] for key in ('stdout', 'stderr', 'merged', 'status'))


def execute(source: str, runtime: Runtime, path: Path) -> Capture:
    return capture(Invocation(argv=[runtime.executables[0], '--vanilla', '--slave', '-e', source], stdin=None), runtime, path)


def observe(case: Case, runtime: Runtime, output: Path) -> Observation:
    direct = execute(case.source, runtime, output / 'direct')
    setup, expression = case.program
    typed = (setup + '\n' if setup else '') + 'oracle_result <- .Internal(withVisible(' + expression + '))\n'
    typed += (ROOT / 'scripts/dynamic_frame_observer.R').read_text()
    observed = execute(typed, runtime, output / 'structural')
    node = b''
    if observed['status'] == 0:
        _, marker, suffix = unhex(observed['stdout']).partition(b'DFNODE1\n')
        assert marker, 'structural marker missing'
        node = marker + suffix
        _ = structure(node)
    prelude = (ROOT / 'scripts/dynamic_frame_conditions.R').read_text()
    annotated = execute(prelude + '\n' + case.source, runtime, output / 'conditions')
    diagnostics: tuple[tuple[str, str], ...] = tuple(
        (match[1].decode().lower(), unhex(match[2].decode()).decode())
        for match in MARKER.finditer(unhex(annotated['stderr'])))
    cleaned = annotated.copy()
    for stream in ('stderr', 'merged'):
        cleaned[stream] = MARKER.sub(b'', unhex(annotated[stream])).hex()
    assert exact(cleaned, direct), 'condition observation changed direct execution'
    return Observation(direct, observed, node, diagnostics)


def fixture_line(case: Case, observation: Observation) -> str:
    setup, expression = case.program
    conditions = '\n'.join(kind + '\t' + payload.encode().hex() for kind, payload in observation.conditions)
    return '\t'.join((case.identifier, setup.encode().hex(), expression.encode().hex(), observation.node.hex(),
                      str(observation.direct['status']), conditions.encode().hex(), case.frame))


def inputs() -> dict[str, str]:
    paths = [*sorted((ROOT / 'scripts').glob('d1_prerequisite_*.py')),
             *sorted((ROOT / 'scripts').glob('dynamic_frame*')),
             ROOT / 'scripts/check-d1-prerequisites.py']
    paths.extend(ROOT / 'crates/r_ust_eval/tests/fixtures' / name for name in (
        'dynamic-frames-r-4.6.0.tsv', 'dynamic-frame-source-r-4.6.0.tsv'))
    return {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in paths if path.is_file()}


def read_fixture(path: Path = FIXTURE) -> list[str]:
    data = path.read_bytes()
    assert sha(data) == FIXTURE_SHA, 'immutable prerequisite fixture changed'
    lines = data.decode().splitlines()
    assert lines[0] == HEADER
    assert len(lines) == len(CASES) + 1
    for case, line in zip(CASES, lines[1:], strict=True):
        fields = line.split('\t')
        assert len(fields) == 7 and fields[0] == case.identifier
        assert tuple(unhex(value).decode() for value in fields[1:3]) == case.program
        if fields[3]:
            _ = structure(unhex(fields[3]))
    return lines[1:]


def run(output: Path, binary: Path | None) -> bool:
    output.mkdir(parents=True, exist_ok=False)
    expected = read_fixture() if binary is not None else None
    before = inputs()
    write_json(output / 'inputs-before.json', before)
    profile = read_reference()[0]['profile']
    runtime = discover(profile, output / 'runtime')
    native = None
    binary_hash = None
    if binary is not None:
        binary_hash = sha(binary.read_bytes())
        native = Runtime((str(binary), str(binary)), runtime.environment, {'binary': binary_hash})
    lines: list[str] = []
    results: list[Result] = []
    for case in CASES:
        directory = output / case.identifier
        first = observe(case, runtime, directory / 'run-1')
        second = observe(case, runtime, directory / 'run-2')
        assert exact(first.direct, second.direct) and exact(first.structural, second.structural)
        assert first.node == second.node and first.conditions == second.conditions
        lines.append(fixture_line(case, first))
        passed = True
        if native is not None:
            actual = execute(case.source, native, directory / 'native')
            condition = Conditions(case.identifier, sha(case.source.encode()), first.conditions, Frame(case.frame))
            passed = native_matches(first.direct, actual, condition)
        results.append(Result(id=case.identifier, reference_two_runs=True, direct_cli=passed if native else 'not-run'))
        print(case.identifier + ': reference agrees; native ' + (str(passed) if native else 'not run'), flush=True)
    fixture = (HEADER + '\n' + '\n'.join(lines) + '\n').encode()
    _ = (output / FIXTURE.name).write_bytes(fixture)
    if binary is not None:
        assert lines == expected, 'fresh reference differs from immutable fixture'
    after = runtime_hashes(runtime.executables, runtime.environment)
    stable = before == inputs() and after == runtime.hashes
    if binary is not None:
        stable = stable and binary_hash == sha(binary.read_bytes())
    direct_pass = all(item['direct_cli'] is True for item in results) if native else False
    report = Report(reference_two_runs=True, count=len(CASES), stable=stable,
        direct_cli=direct_pass if native else 'not-run', native_typed='external Rust consumer required',
        fixture_sha256=sha(fixture), binary_sha256=binary_hash, cases=results, runtime_after=after)
    _ = (output / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    write_json(output / 'output-manifest.json', {str(path.relative_to(output)): sha(path.read_bytes())
        for path in sorted(output.rglob('*')) if path.is_file()})
    assert stable, 'capture inputs changed'
    return direct_pass if native else True
