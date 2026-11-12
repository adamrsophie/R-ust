#!/usr/bin/env python3
import base64
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import subprocess
import sys
import tempfile
from typing import NotRequired, TypedDict


class Program(TypedDict):
    label: str
    source: str
    expected: str
    status: int
    warning: NotRequired[bool]


root = Path(__file__).resolve().parent.parent
binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/r_ust_cli").resolve()
environment = dict(os.environ, R_DEFAULT_PACKAGES="NULL")
version = subprocess.run(["Rscript", "--version"], capture_output=True, text=True, check=True)
if "Rscript (R) version 4.6.0" not in version.stdout + version.stderr:
    raise SystemExit("An independent R 4.6.0 installation is required")
platform = subprocess.run(["Rscript", "--vanilla", "-e", 'cat(R.version$arch,Sys.getlocale("LC_CTYPE"),sep="/")'],
                          env=environment, capture_output=True, text=True, check=True).stdout
probe_source = r'''
use std::io::{self, BufRead};
use r_ust_parse::Parser;
use r_ust_sexp::Sexp;
fn fingerprint(value: &Sexp) -> String {
    match value {
        Sexp::Real(values) if values.len() == 1 => {
            if values[0].is_nan() { "double:nan".into() }
            else { format!("double:{:016x}", values[0].to_bits()) }
        }
        Sexp::Integer(values) if values.len() == 1 => format!("integer:{}", values[0].unwrap()),
        _ => "other".into(),
    }
}
fn main() {
    for source in io::stdin().lock().lines() {
        let source = source.unwrap();
        let eager = Parser.parse_all(&source).map(|result| fingerprint(&result.expressions[0]))
            .unwrap_or_else(|error| format!("{:?}", error.status));
        let mut statements = Parser.parse_iter(&source);
        let incremental = statements.next().unwrap().map(|result| fingerprint(&result.expressions[0]))
            .unwrap_or_else(|error| format!("{:?}", error.status));
        assert_eq!(eager, incremental, "{source}");
        if eager.starts_with("double:") || eager.starts_with("integer:") {
            assert_eq!(statements.consumed_bytes(), source.len(), "{source}");
        }
        println!("{eager}");
    }
}
'''
reference_source = r'''
for(s in readLines(file("stdin"))) {
    out <- withCallingHandlers(tryCatch({
        x <- parse(text=s)[[1]]
        if(is.integer(x)) paste0("integer:",x)
        else if(is.double(x)) {
            if(is.nan(x)) "double:nan"
            else paste0("double:",paste(sprintf("%02x",as.integer(writeBin(x,raw(),size=8L,endian="big"))),collapse=""))
        } else "other"
    },error=function(e)"Error"),warning=function(w)invokeRestart("muffleWarning"))
    cat(out,"\n",sep="")
}
'''
literals = ["0x1F", "0Xff", "0x1e3", "0x1.8", "0x.8", "0x.", "0x.p1", "0x1.p2",
            "0x1p-1023", "0x1p-1024", "0x1p-1074", "0x1p1024", "0x1.fffffffffffffp1023",
            "0x1000000000000080", "0x1000000000000081", "0x1000000000000081p-64",
            "0x1L", "0x7fffffffL", "0x80000000L", "0x1.0L", "0x1.8L", "0x1p2L",
            "0x0p999999999999999999999999", "0x1p999999999999999999999999",
            "0x1p-999999999999999999999999", "1", ".5", "1.25", "1e3", "2L"]
for count in [13, 14, 255, 256, 257, 1000]:
    literals += ["0x" + "f" * count, "0x." + "f" * count,
                 "0x." + "0" * count + "1p1024", "0x0." + "0" * count + "p99999"]
rng = random.Random(460)
for _ in range(1200):
    digits = "".join(rng.choice("0123456789abcdef") for _ in range(rng.randrange(1, 41)))
    if rng.randrange(2):
        point = rng.randrange(len(digits) + 1)
        digits = digits[:point] + "." + digits[point:]
    suffix = rng.choice(["", "p" + str(rng.randrange(-1200, 1201)), "P+" + str(rng.randrange(0, 1201))])
    literals.append(rng.choice(["0x", "0X"]) + digits + suffix + rng.choice(["", "", "L"]))
invalid = ["0x", "0X", "0xp1", "0xL", "0xg", "0x1p", "0x1p+", "0x1p-", "0x.p",
           "0x1.2.3", "0x1p--1", "0x1p1.0", "0x1p1e2", "0x1p1p2", "0x1LL", "0x1p+L"]
literals += invalid
programs: list[Program] = [
    Program(label="inventory", source="0x1F\n", expected="[1] 31\n", status=0),
    Program(label="arithmetic", source="cat(0x1E+2,0x.8+0X1P2,sep=\",|\")\n", expected="32,|4.5", status=0),
    Program(label="types", source='cat(0x1L,typeof(0x1L),typeof(0x1p2),sep=":")\n', expected="1:integer:double", status=0),
    Program(label="zero-dot", source="cat(0x.,0x.p1,sep=\":\")\n", expected="0:0", status=0),
    Program(label="range", source='cat(0x1p-1024==0,is.infinite(0x1p1024),sep=":")\n', expected="TRUE:TRUE", status=0),
    Program(label="nan", source='cat(is.nan(0x.' + '0' * 300 + '1p1024))\n', expected="TRUE", status=0),
    Program(label="signed", source='cat(-0x1.8,+0X1P2,sep=":")\n', expected="-1.5:4", status=0),
    Program(label="closure", source='f<-function(x=0x1.8)x+0x1;cat(f())\n', expected="2.5", status=0),
    Program(label="index", source='x<-c(0x1,0x2,0x3);x[0x2L]<-0x10;cat(sum(x))\n', expected="20", status=0),
    Program(label="loop", source='s<-0;for(i in 0x1L:0x4L)s<-s+i;cat(s)\n', expected="10", status=0),
    Program(label="string", source='cat("0x10")\n', expected="0x10", status=0),
]
programs += [Program(label=f"invalid-{i}", source=f'cat("BEFORE\\n")\n{source}\ncat("AFTER")\n', expected="BEFORE\n", status=1)
             for i, source in enumerate(invalid)]
programs += [Program(label=f"warning-{i}", source=f'cat({source})\n', expected=value, status=0, warning=True)
             for i, (source, value) in enumerate([("0x1.0L", "1"), ("0x1.8L", "1.5"), ("0x80000000L", "2147483648")])]


def recorded(data):
    return dict(text=data.decode("utf-8", errors="replace"), base64=base64.b64encode(data).decode("ascii"))


def run(command, data):
    result = subprocess.run(command, input=data, env=environment, capture_output=True, timeout=30)
    return result, dict(status=result.returncode, stdout=recorded(result.stdout), stderr=recorded(result.stderr))


sources = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
           for path in [*Path("crates").rglob("*.rs"), *Path("crates").glob("*/Cargo.toml"),
                        Path("Cargo.toml"), Path("Cargo.lock"), Path(__file__)]}
results = []
with tempfile.TemporaryDirectory(prefix="r-ust-hex-") as directory:
    directory = Path(directory)
    snapshots = []
    for original in [binary, binary.parent / "Rscript"]:
        snapshot = directory / original.name
        shutil.copy2(original, snapshot)
        snapshots.append(snapshot)
    subprocess.run(["cargo", "build", "--quiet", "-p", "r_ust_parse", "-p", "r_ust_sexp", "--lib"], cwd=root, check=True)
    probe_path = directory / "probe.rs"
    probe_path.write_text(probe_source)
    probe = directory / "probe"
    subprocess.run(["rustc", "--edition=2021", str(probe_path), "-L", f"dependency={root}/target/debug/deps",
                    "--extern", f"r_ust_parse={root}/target/debug/libr_ust_parse.rlib",
                    "--extern", f"r_ust_sexp={root}/target/debug/libr_ust_sexp.rlib", "-o", str(probe)], check=True)
    reference = directory / "reference.R"
    reference.write_text(reference_source)
    data = ("\n".join(literals) + "\n").encode()
    r, rr = run(["Rscript", "--vanilla", str(reference)], data)
    u, uu = run([str(probe)], data)
    assert r.returncode == u.returncode == 0 and not r.stderr and not u.stderr, (rr, uu)
    expected, actual = r.stdout.decode().splitlines(), u.stdout.decode().splitlines()
    assert len(expected) == len(actual) == len(literals)
    parser_results = [dict(source=source, r=r_value, rust=u_value, passed=r_value == u_value)
                      for source, r_value, u_value in zip(literals, expected, actual)]
    retained_probe = Path("target/hex-number-probe")
    shutil.copy2(probe, retained_probe)
    hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in [*snapshots, retained_probe]}
    path = directory / "hex literals.R"
    for mode in ["stdin", "rscript-file", "expression", "rscript-expression"]:
        for case in programs:
            source = case["source"].encode()
            reference_command = ["R", "--vanilla", "--no-echo"]
            native_command = [str(snapshots[0]), "--vanilla", "--no-echo"]
            data = source
            if mode == "rscript-file":
                path.write_bytes(source)
                reference_command = ["Rscript", "--vanilla", str(path)]
                native_command = [str(snapshots[1]), "--vanilla", str(path)]
                data = b""
            elif mode in ["expression", "rscript-expression"]:
                expression = case["source"].replace("\\", "\\\\")
                if mode == "rscript-expression":
                    reference_command = ["Rscript", "--vanilla"]
                    native_command = [str(snapshots[1]), "--vanilla"]
                reference_command += ["-e", expression]
                native_command += ["-e", expression]
                data = b""
            r, r_record = run(reference_command, data)
            u, u_record = run(native_command, data)
            values_match = r.returncode == u.returncode == case["status"] and r.stdout == u.stdout == case["expected"].encode()
            passed = values_match and (bool(r.stderr) and bool(u.stderr) if case["status"] == 1
                                       else r.stderr == u.stderr and bool(r.stderr) == case.get("warning", False))
            results.append(dict(mode=mode, case=case, r=r_record, rust=u_record, passed=passed))
        print(f"Checked {mode}: {len(programs)} programs", flush=True)

Path("target/hex-number-parity.json").write_text(json.dumps(dict(
    r_version=version.stdout + version.stderr, reference_platform=platform, source_sha256=sources,
    binary_sha256=hashes, probe_source=probe_source, reference_source=reference_source,
    parser_streams=dict(r=rr, rust=uu), parser_cases=parser_results, cli_cases=results), indent=2) + "\n")
failed = [row for row in parser_results if not row["passed"]]
cli_failed = [row for row in results if not row["passed"]]
for row in failed[:12] + cli_failed[:12]:
    print(json.dumps(row))
print(f"{len(parser_results) - len(failed)} parser bit/type comparisons pass; {len(failed)} differences")
print(f"{sum(row['passed'] for row in results)} CLI comparisons pass; "
      f"{len(cli_failed)} unexpected differences")
if failed or cli_failed:
    raise SystemExit(1)
