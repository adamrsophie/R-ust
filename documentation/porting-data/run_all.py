import csv, sys
S, binary = sys.argv[1:3]
sys.path.insert(0, S)
from calls import C
from lang import L
from classify import classify
from run_both import run_both

def verdict(r, u):
    if u.startswith("ERROR"):
        if "not found" in u or "parse error" in u or "not implemented" in u:
            return "error"
        return "match" if r.startswith("ERROR") else "error"
    if r.startswith("ERROR"):
        return "value"
    return "match" if r == u else classify("differs", r, u)

# functions
names = sorted({r["name"] for r in csv.DictReader(open(f"{S}/probe.tsv"), delimiter="\t") if r["probe"] == "recognized"})
pairs = [(n, c) for n in names for c in C[n]]
r_out, u_out = run_both(S, binary, [c for _, c in pairs], "fn")
with open(f"{S}/fn_diff.tsv", "w") as f:
    f.write("name\tclass\tcall\tr_out\trust_out\n")
    for (n, c), r, u in zip(pairs, r_out, u_out):
        f.write(f"{n}\t{verdict(r, u)}\t{c}\t{r}\t{u}\n")
# language
r_out, u_out = run_both(S, binary, [c for _, _, c in L], "lang")
with open(f"{S}/lang.tsv", "w") as f:
    f.write("category\tfeature\tclass\tcode\tr_out\trust_out\n")
    for (cat, feat, c), r, u in zip(L, r_out, u_out):
        f.write(f"{cat}\t{feat}\t{verdict(r, u)}\t{c}\t{r}\t{u}\n")
