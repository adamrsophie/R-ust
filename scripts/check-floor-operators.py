#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# ─── How to run ───
# Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh
# Run: uv run scripts/check-floor-operators.py [path/to/r_ust_cli]
# ──────────────────
from __future__ import annotations

import base64
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal, TypedDict, assert_never

Policy = Literal["exact", "messages", "limit"]


@dataclass(frozen=True, slots=True)
class Case:
    source: str
    policy: Policy = "exact"
    messages: tuple[str, ...] = ()


class Result(TypedDict):
    status: int
    stdout: str
    stderr: str


class Comparison(TypedDict):
    mode: str
    source: str
    policy: Policy
    passed: bool
    r: Result
    rust: Result


def recorded(result: subprocess.CompletedProcess[bytes]) -> Result:
    return Result(status=result.returncode, stdout=base64.b64encode(result.stdout).decode("ascii"),
                  stderr=base64.b64encode(result.stderr).decode("ascii"))


def contains_messages(stream: bytes, messages: tuple[str, ...]) -> bool:
    expected = tuple(message.encode() for message in messages)
    if any(stream.count(message) != expected.count(message) for message in expected):
        return False
    position = 0
    for message in expected:
        found = stream.find(message, position)
        if found < 0:
            return False
        position = found + len(message)
    return True


def compare(case: Case, runs: tuple[subprocess.CompletedProcess[bytes], subprocess.CompletedProcess[bytes]]) -> bool:
    reference, native = runs
    same_output = (reference.returncode, reference.stdout) == (native.returncode, native.stdout)
    match case.policy:
        case "exact":
            return same_output and reference.stderr == native.stderr
        case "messages":
            return same_output and all(contains_messages(run.stderr, case.messages) for run in runs)
        case "limit":
            return (reference.returncode, reference.stdout, reference.stderr) != (native.returncode, native.stdout, native.stderr)
    assert_never(case.policy)


RECYCLE: Final = "longer object length is not a multiple of shorter object length"
PRECISION: Final = "probable complete loss of accuracy in modulus"
CASES: Final = (
    Case("cat(c(17 %/% 5,17 %% 5,-7 %% 3),sep=',')"),
    Case("cat(c(-7L %/% 3L,7L %/% -3L,-7L %% 3L,7L %% -3L),sep=',')"),
    Case("cat(typeof(TRUE %% TRUE),typeof(1L %/% 2L),typeof(1L %% 2),sep=',')"),
    Case("cat(is.na(c(1L %% 0L,NA_integer_ %/% 2L,1L %/% 0L)),sep=',')"),
    Case("cat(is.nan(NA_real_ %% 0),is.na(NA_real_ %/% 0),sep=',')"),
    Case("cat(is.nan(NaN %% (NA_real_+1)),is.na((NA_real_+1) %% NaN),sep=',')"),
    Case("cat(1 %% Inf,-1 %% Inf,-1 %/% Inf,sep=',')"),
    Case("cat((-0x1p-1000) %/% 0x1p1000,(0x1p-1000) %/% -0x1p1000,sep=',')"),
    Case("cat(0.3 %/% 0.1,1 %/% 0.1,sep=',')"),
    Case("cat(1+7 %% 3*2,20 %/% 3 %/% 2,2^3 %% 3,sep=',')"),
    Case("cat(1:4 %% 3,sep=',')"),
    Case("cat(length(NULL %% TRUE),typeof(NULL %% TRUE),length(as.double(NULL) %/% 1:3),sep=',')"),
    Case("x<-matrix(1:4,2) %/% c(2,3);cat(x,sep=',')"),
    Case("cat(length(matrix(1:4,2) %% as.double(NULL)))"),
    Case("x<-0L;for(i in 1:9){if(i %% 2L == 0L)x<-x+i};cat(x)"),
    Case("f<-function(x,y=3L)c(x %/% y,x %% y);cat(f(-7L),sep=',')"),
    Case("options(warn=-1);cat(1:3 %% 1:2,sep=',')"),
    Case("cat(1:3 %% 1:2,sep=',')", "messages", (RECYCLE,)),
    Case("options(warn=1);cat(1:3 %/% 1:2,sep=',')", "messages", (RECYCLE,)),
    Case("cat(0x1p54 %% 3)", "messages", (PRECISION,)),
    Case("options(warn=1);cat(c(0x1p54,0x1p54,0x1p54) %% c(3,3))", "messages", (RECYCLE,PRECISION,PRECISION,PRECISION)),
    Case("options(warn=2);cat('BEFORE');1:3 %% 1:2", "messages", (RECYCLE,)),
    Case("options(warn=2);cat('BEFORE');0x1p54 %% 3", "messages", (PRECISION,)),
    Case("cat(is.na(factor(c('a','b')) %% 2),sep=',')", "messages", ("not meaningful for factors",)),
    Case("cat(matrix(1,1) %% 1:3,sep=',')", "messages", ("Recycling array of length 1 in array-vector arithmetic is deprecated.",)),
    Case("matrix(1:4,2) %% matrix(1:4,1)", "messages", ("non-conformable arrays",)),
    Case("matrix(1:4,2) %% 1:6", "messages", ("dims [product 4] do not match the length of object [6]",RECYCLE)),
    Case("cat(1e100 %% 3)", "limit"),
    Case("cat(`%%`(7L,3L))", "limit"),
    Case("cat(names(c(a=1,b=2) %% 2))", "limit"),
    Case("x<-matrix(1:4,2) %/% c(2,3);x", "limit"),
    Case("factor(c('a','b')) %/% factor(c('a','b'),ordered=TRUE)", "limit"),
)


def main() -> None:
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
    reference_r, reference_script = shutil.which("R"), shutil.which("Rscript")
    if not reference_r or not reference_script:
        raise SystemExit("Independent R and Rscript 4.6.0 are required")
    environment = dict(os.environ, LC_ALL="C", R_DEFAULT_PACKAGES="NULL")
    versions = [subprocess.run([path,"--version"],capture_output=True,text=True,check=True)
                for path in (reference_r,reference_script)]
    if any("version 4.6.0" not in run.stdout+run.stderr for run in versions):
        raise SystemExit("Reference R and Rscript must be version 4.6.0")
    sources = {str(path):hashlib.sha256(path.read_bytes()).hexdigest()
               for path in (*Path("crates").rglob("*.rs"),*Path("crates").rglob("*.tsv"),
                            *Path("crates").glob("*/Cargo.toml"),Path("Cargo.toml"),Path("Cargo.lock"),Path(__file__))}
    rows: list[Comparison] = []
    with tempfile.TemporaryDirectory(prefix="r-ust floor operators ") as directory:
        root = Path(directory)
        natives = (root/binary.name,root/"Rscript")
        for original,snapshot in zip((binary,binary.parent/"Rscript"),natives,strict=True):
            _ = shutil.copy2(original,snapshot)
        hashes = {path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path in natives}
        source_file = root/"operators.R"
        for is_script,reference,native in [(False,reference_r,natives[0]),(True,reference_script,natives[1])]:
            for expression in (False,True):
                mode = ("rscript" if is_script else "main") + ("-expression" if expression else "-file")
                prefix = ["--vanilla"] if is_script else ["--vanilla","--no-echo"]
                for case in CASES:
                    _ = source_file.write_text(case.source+"\n")
                    args = ["-e",case.source] if expression else ([str(source_file)] if is_script else ["-f",str(source_file)])
                    runs = tuple(subprocess.run([str(program),*prefix,*args],input=b"",env=environment,
                                                capture_output=True,timeout=20,check=False) for program in (reference,native))
                    r,n = runs
                    passed = compare(case,(r,n))
                    rows.append(Comparison(mode=mode,source=case.source,policy=case.policy,passed=passed,r=recorded(r),rust=recorded(n)))
    assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest for path,digest in sources.items()), "Sources changed during comparison"
    _ = Path("target/floor-operator-parity.json").write_text(json.dumps({"reference_version":[v.stdout+v.stderr for v in versions],
        "source_sha256":sources,"binary_sha256":hashes,"cases":rows},indent=2)+"\n")
    failures = [row for row in rows if not row["passed"]]
    for row in failures:
        print(f"FAIL {row['mode']}: {row['source']}")
    exact = sum(row["policy"]=="exact" for row in rows)
    messages = sum(row["policy"]=="messages" for row in rows)
    limits = sum(row["policy"]=="limit" for row in rows)
    print(f"{exact} exact stream/status comparisons; {messages} message/order comparisons; {limits} known limits; {len(failures)} unexpected differences")
    print("Diagnostic call context is not compared for message cases. Known limits are not passing parity cases.")
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
