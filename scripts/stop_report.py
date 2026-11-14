from __future__ import annotations

import base64
import json
import shutil
from collections.abc import Callable
from dataclasses import asdict
from pathlib import Path
from typing import TypeAlias

from scripts.stop_capture import (
    FIXTURE_SHA,
    MODES,
    Invocation,
    Run,
    digest,
    encode_bytes,
    matches,
)
from scripts.stop_reference import ARTIFACTS, FIXTURE, PROFILE, ROOT, read_reference

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]


def json_decoder() -> Callable[[str], Json]:
    return json.loads


def source_inputs() -> tuple[Path, ...]:
    return tuple(
        sorted(
            (
                *ROOT.glob("crates/**/*.rs"),
                *ROOT.glob("crates/**/*.tsv"),
                *ROOT.glob("crates/**/Cargo.toml"),
                ROOT / "Cargo.toml",
                ROOT / "Cargo.lock",
                ROOT / "scripts/check-stop.py",
                ROOT / "scripts/stop_reference.py",
                ROOT / "scripts/stop_capture.py",
                Path(__file__),
            )
        )
    )


def reference_inputs() -> tuple[Path, ...]:
    return tuple(Path(shutil.which(name) or name).resolve() for name in ("R", "Rscript"))


def read_run(value: Json) -> Run:
    assert isinstance(value, dict)
    request = value["invocation"]
    assert isinstance(request, dict)
    command, stdin = request["command"], request["stdin"]
    assert isinstance(command, list)
    assert isinstance(stdin, str)
    arguments: list[str] = []
    for argument in command:
        assert isinstance(argument, str)
        arguments.append(argument)
    assert arguments
    status, stdout, stderr = value["status"], value["stdout"], value["stderr"]
    assert type(status) is int
    assert isinstance(stdout, str)
    assert isinstance(stderr, str)
    stdout_base64, stderr_base64 = value["stdout_base64"], value["stderr_base64"]
    assert isinstance(stdout_base64, str)
    assert isinstance(stderr_base64, str)
    assert encode_bytes(stdout.encode()) == stdout_base64
    assert encode_bytes(stderr.encode()) == stderr_base64
    return Run(
        Invocation(tuple(arguments), base64.b64decode(stdin, validate=True)),
        status,
        stdout,
        stderr,
        stdout_base64,
        stderr_base64,
    )


def report_check(directory: Path = ARTIFACTS / "compare") -> int:
    """Recompute every saved outcome and require an exact, fresh input manifest."""
    report = directory / "report.json"
    data = json_decoder()(report.read_text())
    assert isinstance(data, dict)
    assert data["fixture_sha256"] == FIXTURE_SHA == digest(FIXTURE)
    assert data["stable"] is True
    assert data["baseline"] is False
    assert data["reference_profile"] == [PROFILE, PROFILE]
    expected_summary = [f"report\t{report}\t{digest(report)}"]
    for field, paths in (("source_sha256", source_inputs()), ("reference_sha256", reference_inputs())):
        recorded = data[field]
        assert isinstance(recorded, dict)
        assert recorded == {str(path): digest(path) for path in paths}, field
    binaries = data["binary_sha256"]
    assert isinstance(binaries, dict)
    assert len(binaries) == 2
    assert any(Path(path).name == "Rscript" for path in binaries)
    assert len({Path(path).parent for path in binaries}) == 1
    for field in ("source_sha256", "binary_sha256", "reference_sha256"):
        manifest = data[field]
        assert isinstance(manifest, dict)
        for path, value in manifest.items():
            assert isinstance(value, str)
            assert digest(Path(path)) == value, path
            expected_summary.append(f"input\t{path}\t{value}")
    cases = data["cases"]
    assert isinstance(cases, list)
    rows = read_reference()
    expected = [(mode, row) for mode in MODES for row in rows]
    for case, (mode, row) in zip(cases, expected, strict=True):
        assert isinstance(case, dict)
        assert case["mode"] == mode
        assert case["reference"] == asdict(row)
        r, native, r_merged, native_merged = (
            read_run(case[key]) for key in ("r", "native", "r_merged", "native_merged")
        )
        assert matches(row, r, r_merged, reference=True), (mode, row.spec.label, "reference")
        supported = True
        passed = supported and matches(row, native, native_merged, reference=False)
        assert case["supported"] is supported
        assert case["passed"] is passed
        assert case["exact_streams"] is (
            (r.stdout_base64, r.stderr_base64) == (native.stdout_base64, native.stderr_base64)
        )
        assert case["exact_merged"] is (r_merged.stdout_base64 == native_merged.stdout_base64)
        assert not supported or passed, (mode, row.spec.label, "native")
        expected_summary.append(
            f"row\t{mode}/{row.identifier}/{row.spec.label}\t" + ("PASS" if supported else "LIMITED")
        )
    assert (directory / "summary.tsv").read_text().splitlines() == expected_summary
    print("Report hashes and all 605 outcomes verified; 605 PASS, 0 LIMITED")
    return 0
