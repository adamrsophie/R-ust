#!/usr/bin/env python3
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from typing import TypedDict


class Recorded(TypedDict):
    text: str
    base64: str


class RunResult(TypedDict):
    status: int
    stdout: Recorded
    stderr: Recorded


binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
environment = os.environ.copy()
environment["R_DEFAULT_PACKAGES"] = "NULL"


def recorded(data: bytes) -> Recorded:
    return Recorded(text=data.decode("utf-8", errors="replace"),
                base64=base64.b64encode(data).decode("ascii"))


def run(command: list[str]) -> RunResult:
    result = subprocess.run(command, capture_output=True, env=environment, timeout=20)
    return RunResult(status=result.returncode, stdout=recorded(result.stdout),
                stderr=recorded(result.stderr))


sources = [r"cat('a\0101b')", r"cat('a\x41b')", r"cat('a\0b')",
           r"cat('a\nb')", r"cat('a\\nb')", r"cat('a\tb')", r"cat('a\\tb')",
           r"cat('a\rb')", r"cat('a\ab')", r"cat('a\bb')", r"cat('a\fb')",
           r"cat('a\vb')", r"cat('\xC3\xA9')", "1\n\n", r"1\n2",
           r'options(echo=TRUE);cat("VALUE\n")', r"cat('a~n~b~t~c~+~d')"]
payloads = ["a" + rf"\x{value:02x}" + "b" for value in range(128)]
payloads += ["a" + rf"\0{value:03o}" + "b" for value in [0, 7, 8, 9, 10, 13, 32, 65, 127, 256, 383]]
payloads += [r"a\eb", r"a\Eb", r"a\101b", r"a\u0061b", r"a\zb", r"a\xb",
             r"a\x", r"a\\nb", r"a\\cb", "a\\", "α β\n\t", "1\n\n",
             r"a\xC3\xA9b", r"a\0303\0251b", "a~n~b~t~c~+~d"]
results = []
limits = []
probe = "a<-commandArgs(FALSE);writeLines(a[length(a)]);q('no')"
source_hashes = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                              Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
with tempfile.TemporaryDirectory(prefix="r-ust-launcher-") as directory:
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = Path(directory) / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in snapshots}
    for name, reference, native in [
        ("R", ["R", "--vanilla", "--no-echo"], [str(snapshots[0]), "--no-echo"]),
        ("Rscript", ["Rscript", "--vanilla"], [str(snapshots[1])]),
    ]:
        preflight = run(reference + ["-e", probe, "-e", "1"])
        if preflight["status"] != 0 or preflight["stdout"]["text"] != "1\n":
            raise SystemExit(f"Reference metadata probe failed: {preflight}")
        for lane, codes in [("execution", sources), ("metadata", payloads)]:
            for source in codes:
                prefix = ["-e", probe] if lane == "metadata" else []
                args = [*prefix, "-e", source]
                r, u = run(reference + args), run(native + args)
                results.append(dict(frontend=name, lane=lane, source=source, r=r, rust=u,
                                    passed=(r == u and r["status"] == 0)))
            print(f"Checked {name} {lane}", flush=True)
        args = ["-e", "x<-1", "-e", r"x\n+2", "--args", r"literal\n", ""]
        r, u = run(reference + args), run(native + args)
        results.append(dict(frontend=name, lane="repeated", args=args, r=r, rust=u,
                            passed=(r == u and r["status"] == 0)))
        for source in [r"cat('a\\ab')", r"cat('a\\vb')", r"cat('a\\x41b')"]:
            r, u = run(reference + ["-e", source]), run(native + ["-e", source])
            results.append(dict(frontend=name, lane="R string parser escapes", source=source,
                                r=r, rust=u, passed=(r == u and r["status"] == 0)))
        for source in [r"cat('a\xffb')", r"cat('a\cb')"]:
            u = run(native + ["-e", source])
            expected_status = 2 if name == "R" else 1
            expected_error = ("-e launcher escapes produced non-UTF-8 source\n" if r"\xff" in source
                              else "-e launcher escape \\c is not supported\n")
            limits.append(dict(frontend=name, kind="unsupported launcher bytes or truncation",
                               source=source, reference_run=False, rust=u,
                               limit_observed=(u["status"] == expected_status
                                               and not u["stdout"]["text"]
                                               and u["stderr"]["text"] == expected_error)))
        if name == "R":
            for source in ["", "-1", r"\00551"]:
                r, u = run(reference + ["-e", source]), run(native + ["-e", source])
                results.append(dict(frontend=name, lane="CLI argument errors", source=source,
                                    r=r, rust=u, passed=(r == u and r["status"] == 1)))

artifact = Path("target/cli-launcher-parity.json")
artifact.write_text(json.dumps(dict(r_version=version.stdout + version.stderr,
                                    reference_default_packages="NULL", source_sha256=source_hashes,
                                    binary_sha256=hashes, cases=results, known_limits=limits), indent=2) + "\n")
failed = [row for row in results if not row["passed"]]
failed += [row for row in limits if not row["limit_observed"]]
for row in failed[:10]:
    print(json.dumps(row))
print(f"{len(results)} launcher comparisons; {len(limits)} excluded limits; {len(failed)} unexpected differences")
print("Passing cases compare exact stdout, stderr and status. Reference crash cases are not run.")
if failed:
    raise SystemExit(1)
