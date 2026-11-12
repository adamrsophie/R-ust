"""Compare visibility and both console streams against installed reference R."""
import csv
import json
import subprocess
import sys

S, binary = sys.argv[1:3]
sys.path.insert(0, S)
from classify import classify

codes = [
    "1", "NULL", "{}", "if(FALSE) 1", "if(TRUE) 1", "while(FALSE) 1",
    "set.seed(1)", "withVisible(set.seed(1))$visible",
    "x<-1", "(x<-1)", "{x<-1}", "({x<-1})", "invisible()", "invisible(1)",
    "(invisible(1))", "invisible(invisible(1))", "withVisible(invisible(1))",
    "withVisible()", "invisible(bogus=1)", "invisible(1,2)",
    "f<-function(){x<-1}; f()", "f<-function(){x<-1}; (f())",
    "(function()return())()", "(function()return(invisible(1)))()",
    "(function(){(return(invisible(1)));2})()",
    "(function(x)x)(invisible(1))", "(function(x){x;x})(invisible(1))",
    "(function(x=invisible(1))x)()", "(function(x)withVisible(x)$visible)(invisible(1))",
    "with(list(a=1),invisible(a))", "withVisible(sapply(1:2,invisible))$visible",
    "withVisible(for(i in 1:2) i)$visible", "withVisible(if(FALSE) 1)$visible",
    "withVisible(invisible(FALSE)&&1)$visible", "withVisible(1+invisible(2))$visible",
    "withVisible(c(invisible(1)))$visible", "withVisible((invisible(1)))$visible",
    "print(1)", "print(NULL)", 'print("x",quote=FALSE)', "print(print(1))",
    "(print(1))", "invisible(print(1))", "withVisible(print(1))$visible",
    "for(i in 1:3) print(i)", "f<-function(){print(1);invisible(2)};f()",
    "invisible(lapply(1:2,print))", "print(1);missing_value",
    'cat("a","b",sep="-");print(3)', 'cat(c("a","b"),sep="\\n")',
    'cat("x",sep="\\n")', 'cat(NULL,sep="\\n")', "cat()", "cat(1:3)",
    'cat("a","b",sep="x\\ny")', 'cat("err",file=2)',
    'for(i in 1:2)cat(i,"\\n",sep="")',
    'writeLines(c("a","b"))', 'writeLines(c("a","b"),sep="|")',
    'writeLines(c("a","b"),con=2,sep="|")', 'writeLines(c("a","b"),"",":")',
    'writeLines(c(NA_character_,"b"))', "writeLines(1)",
    'message("a",1)', 'message("a\\n")', 'message("a",appendLF=FALSE)', "message()",
    'message(factor(c("b","a",NA)))', 'cat(factor(c("b","a",NA)))',
    'cat("a");message("b");print(3)',
    "a<-1:3;(a[2]<-9);a", "a<-list(x=1);withVisible(a$x<-2)$visible;a$x",
    '(x)<-1;x', '(sum)(1:3)', 'sapply(1:2,(sqrt))',
]
failed = []
unsupported = []
known_unsupported = {'writeLines(c("a","b"),"",":")'}
with open(f"{S}/output.tsv", "w") as stream:
    writer = csv.writer(stream, delimiter="\t", quoting=csv.QUOTE_NONE, quotechar=None, lineterminator="\n")
    writer.writerow(["code", "class", "r_stdout", "rust_stdout", "r_stderr", "rust_stderr"])
    for code in codes:
        r = subprocess.run(["Rscript", "--vanilla", "-e", "options(digits=12);" + code], capture_output=True, text=True, timeout=15)
        u = subprocess.run([binary, "-e", code], capture_output=True, text=True, timeout=15)
        result = classify("match" if r.stdout == u.stdout else "differs", r.stdout, u.stdout)
        if r.returncode != 0 and u.returncode != 0 and result in {"match", "format"}:
            result = "error-match"
        elif (r.returncode != 0) != (u.returncode != 0) or r.stderr != u.stderr:
            result = "value"
        if code in known_unsupported and r.returncode == 0 and u.returncode != 0 and "not implemented" in u.stderr:
            result = "unsupported"
            unsupported.append(code)
        writer.writerow([code, result, *(json.dumps(text, ensure_ascii=False) for text in [r.stdout, u.stdout, r.stderr, u.stderr])])
        if result not in {"match", "format", "error-match", "unsupported"}:
            failed.append((code, r.stdout, u.stdout, r.stderr, u.stderr))
print(f"{len(codes)} console and visibility probes; {len(unsupported)} unsupported; {len(failed)} mismatches")
for failure in failed:
    print(failure)
if failed:
    raise SystemExit(1)
