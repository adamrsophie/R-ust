# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Typed public D1 records and the untrusted JSON boundary.

Validate through: python3 scripts/dynamic_frame_reference.py self-check
"""
from __future__ import annotations

import hashlib
import json
import re
from typing import Final, Literal, TypeAlias, TypeGuard, TypedDict, cast

class OptimizedPythonError(RuntimeError):
    """Optimized Python would disable required reference validation."""


if not __debug__:
    raise OptimizedPythonError('dynamic frame verification requires non-optimized Python')

POINTER: Final = re.compile(rb'(?<=<environment: )0x[0-9a-fA-F]+(?=>)')
MODES: Final = ('expression', 'file', 'stdin', 'rscript-file', 'rscript-expression', 'typed')
Scope = Literal['cli', 'sdk-reference-only']
Contract = Literal['supported', 'pending-bytecode']
JsonValue: TypeAlias = str | int | float | bool | None | list['JsonValue'] | dict[str, 'JsonValue']


class Invocation(TypedDict):
    argv: list[str]
    stdin: str | None


class Capture(TypedDict):
    stdout: str
    stderr: str
    merged: str
    status: int
    invocation: Invocation


class Profile(TypedDict):
    environment: dict[str, str]
    hashes: dict[str, str]
    version: str
    observer_sha256: str


class Row(TypedDict):
    id: str
    scope: Scope
    contract: Contract
    setup: bytes
    expression: bytes
    source: bytes
    typed_source: bytes
    raw: dict[str, dict[str, Capture]]
    node: bytes
    profile: Profile


class Pin(TypedDict):
    fixture: str
    count: int
    source: str
    typed_source: str
    raw: str


class CaptureStatus(TypedDict):
    separate: int
    merged: int


class RuntimeAttestation(TypedDict):
    executables: tuple[str, str]
    hashes: dict[str, str]
    environment: dict[str, str]
    source_hashes: str


class Verification(TypedDict):
    result: Literal['PASS']
    original: int
    cli: int
    sdk_reference_only: int
    supplemental: int
    categories: dict[str, int]
    pending_bytecode: list[str]
    raw_byte_comparison: str
    observations: int
    source_hashes_reverified: bool


JsonDocument: TypeAlias = dict[str, str] | Invocation | Profile | CaptureStatus | RuntimeAttestation | Verification


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unhex(text: str) -> bytes:
    assert re.fullmatch(r'(?:[0-9a-f]{2})*', text), 'invalid hex'
    return bytes.fromhex(text)


def mapping(value: JsonValue) -> dict[str, JsonValue]:
    assert isinstance(value, dict), 'JSON object required'
    return value.copy()


def sequence(value: JsonValue) -> TypeGuard[list[JsonValue]]:
    return isinstance(value, list)


def string(value: JsonValue) -> str:
    assert isinstance(value, str), 'JSON string required'
    return value


def _pairs(values: list[tuple[str, JsonValue]]) -> dict[str, JsonValue]:
    result: dict[str, JsonValue] = {}
    for key, value in values:
        assert key not in result, 'duplicate JSON key'
        result[key] = value
    return result


def decode(data: bytes) -> dict[str, JsonValue]:
    return mapping(cast(JsonValue, json.loads(data, object_pairs_hook=_pairs)))


def parse_capture(value: JsonValue) -> Capture:
    fields = mapping(value)
    assert set(fields) == {'stdout', 'stderr', 'merged', 'status', 'invocation'}
    call = mapping(fields['invocation'])
    assert set(call) == {'argv', 'stdin'}
    arguments = call['argv']
    assert sequence(arguments) and arguments
    argv = [string(argument) for argument in arguments]
    incoming = call['stdin']
    assert incoming is None or isinstance(incoming, str)
    status = fields['status']
    assert type(status) is int and status in (0, 1)
    stdout, stderr, merged = (string(fields[name]) for name in ('stdout', 'stderr', 'merged'))
    for stream in (stdout, stderr, merged):
        _ = unhex(stream)
    return Capture(stdout=stdout, stderr=stderr, merged=merged, status=status,
                   invocation=Invocation(argv=argv, stdin=incoming))


def parse_raw(data: bytes) -> dict[str, dict[str, Capture]]:
    fields = decode(data)
    assert set(fields) == {'run-1', 'run-2'}
    result: dict[str, dict[str, Capture]] = {}
    for name, value in fields.items():
        modes = mapping(value)
        assert set(modes) == set(MODES)
        result[name] = {mode: parse_capture(capture) for mode, capture in modes.items()}
    return result


def parse_profile(data: bytes) -> Profile:
    fields = decode(data)
    assert set(fields) == {'environment', 'hashes', 'version', 'observer_sha256'}
    environment = {key: string(value) for key, value in mapping(fields['environment']).items()}
    assert environment == dict(LANG='C', LC_ALL='C', R_DEFAULT_PACKAGES='base', R_ENABLE_JIT='0')
    hashes = {key: string(value) for key, value in mapping(fields['hashes']).items()}
    assert {'R', 'Rscript', 'bin/exec/R', 'libR.dylib'} <= hashes.keys()
    observer = string(fields['observer_sha256'])
    assert all(re.fullmatch('[0-9a-f]{64}', value) for value in (*hashes.values(), observer))
    version = string(fields['version'])
    assert version.startswith('R version 4.6.0 ')
    return Profile(environment=environment, hashes=hashes, version=version, observer_sha256=observer)


def normalized(data: bytes) -> bytes:
    return POINTER.sub(b'<POINTER>', data)


def equal(left: Capture, right: Capture) -> None:
    assert left['status'] == right['status'], 'status differs'
    assert normalized(unhex(left['stdout'])) == normalized(unhex(right['stdout'])), 'stdout differs'
    assert normalized(unhex(left['stderr'])) == normalized(unhex(right['stderr'])), 'stderr differs'
    assert normalized(unhex(left['merged'])) == normalized(unhex(right['merged'])), 'merged differs'
