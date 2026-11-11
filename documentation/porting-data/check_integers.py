"""Compare integer storage and values against installed reference R."""
import csv
import itertools
import sys

S, binary = sys.argv[1:3]
sys.path.insert(0, S)
from classify import classify
from run_both import run_both

values = ["0L", "1L", "-1L", "2147483647L", "-2147483647L", "NA_integer_", "TRUE", "FALSE", "NA", "1.5", "NA_real_"]
expressions = [f"({a}) {op} ({b})" for op in ["+", "-", "*", "/", "^"] for a, b in itertools.product(values, repeat=2)]
expressions += ["1L", "1.0L", "1e2L", "1.5L", "2147483648L", "-2147483648L", "1:3", "3:1", "1:3.5", "1.5:3", "1:2.99999999", "1e20:1e20", "2147483646:2147483647.5", "(-2147483647):(-2147483648)", "seq(3)", "seq(1,3,by=1)", "seq(1L,3L,by=1L)", "seq(1L,1L,by=0)", "seq_len(3.9)", "seq_len(0)", "seq_along(NULL)", "NULL+TRUE", "NULL+NULL", "as.integer(c(-2147483648,2147483647.9))", "seq(-1e308,1e308,by=1e308)"]
for function in ["sum", "prod", "min", "max", "range", "median", "mean", "cumsum", "cumprod", "cummin", "cummax", "diff"]:
    expressions += [f"{function}({data})" for data in ["1:3", "TRUE", "c(1L,NA_integer_)", "c(2147483647L,1L,-1L)", "as.integer(NULL)"]]
expressions += ["c(1L,TRUE,NA_integer_)", "sapply(1:3,function(x) x)", "unlist(list(1L,2L))", "append(1:3,4L)", "replace(1:3,2,9)", "rep(1L,3)", "ifelse(c(TRUE,FALSE,NA),1:3,10:12)", "sort(3:1)", "pmin(1:3,2L)", "pmax(TRUE,NA)"]
codes = [f"x <- {expression}; typeof(x); if(length(x)>0) x" for expression in expressions]
codes += ["typeof(matrix(1:4,2))", "typeof(t(matrix(1:4,2)))", "typeof(cbind(1:2,3:4))", "typeof(rbind(1:2,3:4))", "x<-1:3; x[1]<-99; typeof(x); x", "x<-1:3; x[1]<-9L; typeof(x); x", 'x<-1:3; x[1]<-"x"; typeof(x); x']
r_out, u_out = run_both(S, binary, codes, "integers")
failed = []
with open(f"{S}/integers.tsv", "w") as stream:
    writer = csv.writer(stream, delimiter="\t", quoting=csv.QUOTE_NONE, quotechar=None, lineterminator="\n")
    writer.writerow(["code", "class", "r_out", "rust_out"])
    for code, r, u in zip(codes, r_out, u_out):
        result = classify("match" if r == u else "differs", r, u)
        writer.writerow([code, result, r, u])
        if result not in {"match", "format"}:
            failed.append((code, r, u))
print(f"{len(codes)} integer probes; {len(failed)} mismatches")
for failure in failed:
    print(failure)
if failed:
    raise SystemExit(1)
