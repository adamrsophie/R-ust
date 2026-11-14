# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Frozen GNU R invocation, runtime attestation, and raw capture.

Run: python3 scripts/dynamic_frame_fixture.py verify
"""
from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import shutil
import subprocess

from dynamic_frame_records import Capture, CaptureStatus, Invocation, JsonDocument, Profile, RuntimeAttestation, sha


@dataclass(frozen=True, slots=True)
class Runtime:
    executables: tuple[str, str]
    environment: dict[str, str]
    hashes: dict[str, str]


def write_json(path: Path, value: JsonDocument) -> None:
    _ = path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def capture(request: Invocation, runtime: Runtime, destination: Path) -> Capture:
    destination.mkdir(parents=True)
    incoming = request['stdin']
    runs: list[subprocess.CompletedProcess[bytes]] = []
    for merged in (False, True):
        runs.append(subprocess.run(request['argv'], input=incoming.encode() if incoming is not None else None,
            env=runtime.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT if merged else subprocess.PIPE,
            timeout=30, check=False))
    stdout, stderr, merged = runs[0].stdout, runs[0].stderr, runs[1].stdout
    assert stderr is not None
    for name, data in (('stdout', stdout), ('stderr', stderr), ('merged', merged)):
        _ = (destination / name).write_bytes(data)
    write_json(destination / 'status.json', CaptureStatus(separate=runs[0].returncode, merged=runs[1].returncode))
    write_json(destination / 'invocation.json', request)
    assert runs[0].returncode == runs[1].returncode
    return Capture(stdout=stdout.hex(), stderr=stderr.hex(), merged=merged.hex(),
                   status=runs[0].returncode, invocation=request)


def runtime_hashes(executables: tuple[str, str], environment: dict[str, str]) -> dict[str, str]:
    home = subprocess.check_output([executables[0], 'RHOME'], env=environment, timeout=30).decode().strip()
    paths = {'R': Path(executables[0]), 'Rscript': Path(executables[1]),
             'bin/exec/R': Path(home) / 'bin/exec/R', 'libR.dylib': Path(home) / 'lib/libR.dylib'}
    return {label: sha(path.read_bytes()) for label, path in paths.items()}


def discover(profile: Profile, output: Path) -> Runtime:
    r, rscript = shutil.which('R'), shutil.which('Rscript')
    assert r is not None and rscript is not None, 'GNU R and Rscript must be on PATH'
    environment = dict(os.environ, **profile['environment'])
    executables = (r, rscript)
    hashes = runtime_hashes(executables, environment)
    assert hashes == {label: profile['hashes'][label] for label in hashes}, 'GNU R runtime hashes differ'
    runtime = Runtime(executables, environment, hashes)
    version = capture(Invocation(argv=[r, '--version'], stdin=None), runtime, output / 'version')
    assert version['status'] == 0 and bytes.fromhex(version['stdout']).decode() == profile['version']
    startup = (
        'stopifnot(identical(as.character(getRversion()),"4.6.0"),'
        'identical(Sys.getlocale("LC_CTYPE"),"C"),'
        'identical(search(),c(".GlobalEnv","Autoloads","package:base")),'
        'identical(compiler::enableJIT(-1L),0L))\ncat("PROFILE_OK\\n")\n'
    )
    _ = (output / 'startup.R').write_text(startup)
    calibration = capture(Invocation(argv=[r, '--vanilla', '--slave', '-e', startup], stdin=None),
                          runtime, output / 'startup')
    assert calibration['status'] == 0 and bytes.fromhex(calibration['stdout']) == b'PROFILE_OK\n'
    assert calibration['stderr'] == ''
    write_json(output / 'runtime-before.json', RuntimeAttestation(executables=executables, hashes=hashes,
        environment=profile['environment'], source_hashes='Published provenance facts; primary source files are not required or reverified.'))
    return runtime


def invocation(mode: str, source: Path, runtime: Runtime) -> Invocation:
    r, rscript = runtime.executables
    text = source.read_text()
    if mode == 'expression':
        return Invocation(argv=[r, '--vanilla', '--slave', '-e', text], stdin=None)
    if mode in ('file', 'typed', 'structural'):
        return Invocation(argv=[r, '--vanilla', '--slave', '-f', str(source)], stdin=None)
    if mode == 'stdin':
        return Invocation(argv=[r, '--vanilla', '--slave'], stdin=text)
    if mode == 'rscript-file':
        return Invocation(argv=[rscript, '--vanilla', str(source)], stdin=None)
    assert mode == 'rscript-expression'
    return Invocation(argv=[rscript, '--vanilla', '-e', text], stdin=None)
