"""Compare supported operator combinations against the installed reference R."""
import csv
import itertools
import sys

S, binary = sys.argv[1:3]
sys.path.insert(0, S)
from classify import classify
from run_both import run_both

ops = ["+", "-", "*", "/", "^", "<", "<=", ">", ">=", "==", "!=", "&", "|", "&&", "||", "%in%"]
prefixes = ["-", "+", "!", "!-", "-!"]
codes = [f"2 {a} 3 {b} 4" for a, b in itertools.product(ops, repeat=2)]
codes += [f"{prefix}2 {op} 3" for prefix in prefixes for op in ops]
codes += [f"2 {op} {prefix}3" for prefix in prefixes for op in ops]
r_out, u_out = run_both(S, binary, codes, "operators")
failed = []
with open(f"{S}/operators.tsv", "w") as stream:
    writer = csv.writer(stream, delimiter="\t", quoting=csv.QUOTE_NONE, lineterminator="\n")
    writer.writerow(["code", "class", "r_out", "rust_out"])
    for code, r, u in zip(codes, r_out, u_out):
        if r.startswith("ERROR") and u.startswith("ERROR"):
            result = "error-match"
        else:
            result = classify("match" if r == u else "differs", r, u)
        writer.writerow([code, result, r, u])
        if result not in {"match", "format", "error-match"}:
            failed.append((code, r, u))
print(f"{len(codes)} operator probes; {len(failed)} mismatches")
for failure in failed:
    print(failure)
if failed:
    raise SystemExit(1)
