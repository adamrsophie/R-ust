"""Attest immutable verifier inputs and snapshot the supplied CLI executables."""
from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path
import shutil

from dynamic_frame_capture import Runtime, capture, discover, invocation, runtime_hashes
from dynamic_frame_records import Profile, parse_profile, sha
from eval_runtime_records import ROOT


@dataclass(frozen=True, slots=True)
class Session:
    output: Path
    reference: Runtime
    native: Runtime


def profile() -> Profile:
    path = ROOT / "scripts/eval-runtime-profile.json"
    data = path.read_bytes()
    assert sha(data) == "fc61245db52efa6a33bac1f24336a5ec733f1136f48240d08afb93d9f623a413", "immutable R profile changed"
    value = parse_profile(data)
    observer = ROOT / "scripts/eval-runtime-observer.R"
    assert sha(observer.read_bytes()) == value["observer_sha256"], "observer provenance differs"
    return value


def inputs() -> tuple[Path, ...]:
    return tuple(sorted({*ROOT.glob("crates/**/*.rs"), *ROOT.glob("crates/**/*.tsv"),
        *ROOT.glob("crates/**/Cargo.toml"), *(path for path in (ROOT / "Cargo.toml", ROOT / "Cargo.lock") if path.is_file()),
        *ROOT.glob("scripts/eval_runtime_*.py"), *ROOT.glob("scripts/eval-runtime-*"),
        ROOT / "scripts/check-eval-runtime.py", ROOT / "scripts/dynamic_frame_capture.py",
        ROOT / "scripts/dynamic_frame_records.py"}))


def hashes(paths: tuple[Path, ...]) -> dict[str, str]:
    return {str(path): sha(path.read_bytes()) for path in paths}


def create(binary: Path, output: Path) -> Session:
    selected_profile = profile()
    executables = binary, binary.parent / "Rscript"
    assert all(path.is_file() for path in executables), "both native CLI executables required"
    discovered = discover(selected_profile, output / "reference-runtime")
    environment = {key: value for key, value in discovered.environment.items()
                   if key in ("PATH", "HOME", "TMPDIR", "TMP", "TEMP", "SYSTEMROOT")}
    environment.update(selected_profile["environment"])
    reference = Runtime(discovered.executables, environment, discovered.hashes)
    assert runtime_hashes(reference.executables, environment) == reference.hashes
    startup = output / "reference-runtime/startup.R"
    probe = capture(invocation("file", startup, reference), reference, output / "effective-environment-profile")
    assert probe["status"] == 0 and bytes.fromhex(probe["stdout"]) == b"PROFILE_OK\n" and probe["stderr"] == ""
    (output / "bin").mkdir()
    snapshots = output / "bin/r_ust_cli", output / "bin/Rscript"
    for source, target in zip(executables, snapshots, strict=True):
        _ = shutil.copy2(source, target)
        assert sha(source.read_bytes()) == sha(target.read_bytes())
    native = Runtime((str(snapshots[0]), str(snapshots[1])), reference.environment, hashes(snapshots))
    _ = (output / "environment.json").write_text(json.dumps(environment, indent=2) + "\n")
    return Session(output, reference, native)
