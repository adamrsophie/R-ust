"""Compare NA/NaN propagation against the installed reference R."""
import csv
import itertools
import sys

S, binary = sys.argv[1:3]
sys.path.insert(0, S)
from classify import classify
from run_both import run_both

values = ["NA_real_", "NaN", "0", "1", "Inf"]
pairs = list(itertools.product(values, repeat=2))
codes = [f"{a} {op} {b}" for op in ["+", "-", "*", "/", "^"] for a, b in pairs]
for function in ["sum", "prod", "mean", "min", "max", "range", "median", "var", "sd", "cumsum", "cumprod", "cummin", "cummax"]:
    codes += [f"{function}(c({a}, {b}))" for a, b in pairs]
for function in ["pmin", "pmax"]:
    codes += [f"{function}({a}, {b}, na.rm={remove})" for a, b in pairs for remove in ["TRUE", "FALSE"]]
for function in ["sqrt", "abs", "sin", "log", "exp", "gamma", "lgamma", "digamma", "trigamma", "sign", "round", "dnorm", "pnorm", "qnorm"]:
    codes += [f"{function}({value})" for value in ["NA_real_", "NaN"]]
codes += [
    "c(NA_real_, NaN)",
    "is.nan(c(NA, NA_integer_, NA_real_, NaN))",
    "is.na(c(NA_character_, NA_real_, NaN))",
    "as.character(c(NA_real_, NaN))",
    'as.double(c(NA_character_, "NaN", "bad"))',
    "match(c(NA_real_, NaN), c(NaN, NA_real_))",
    "unique(c(NA_real_, NaN, NA_real_, NaN))",
    "duplicated(c(NA_real_, NaN, NA_real_, NaN))",
    "c(identical(NA_real_, NaN), identical(NA_real_, NA_real_))",
    "x <- c(1, NaN); x[c(2, 3, NA)]",
    "x <- c(1, NaN); x[4] <- 9; x",
    "rowSums(matrix(c(NA_real_, 1, 2, 3), 2))",
    "colMeans(matrix(c(NA_real_, 1, 2, 3), 2))",
    "mean(c(1e308, 1e308))",
]
r_out, u_out = run_both(S, binary, codes, "missing")
failed = []
with open(f"{S}/missing.tsv", "w") as stream:
    writer = csv.writer(stream, delimiter="\t", quoting=csv.QUOTE_NONE, quotechar=None, lineterminator="\n")
    writer.writerow(["code", "class", "r_out", "rust_out"])
    for code, r, u in zip(codes, r_out, u_out):
        result = classify("match" if r == u else "differs", r, u)
        writer.writerow([code, result, r, u])
        if result not in {"match", "format"}:
            failed.append((code, r, u))
print(f"{len(codes)} missing-value probes; {len(failed)} mismatches")
for failure in failed:
    print(failure)
if failed:
    raise SystemExit(1)
