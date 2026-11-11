import csv, re, sys
src_path, inv, probe, out = sys.argv[1:5]
lines = open(src_path).read().split("\n")
fn_re = re.compile(r'^(\s*)(?:pub(?:\([^)]*\))? )?fn (\w+)')
bodies = {}
i = 0
while i < len(lines):
    m = fn_re.match(lines[i])
    if m:
        indent, name = m.group(1), m.group(2)
        j = i + 1
        while j < len(lines) and lines[j] != indent + "}":
            j += 1
        bodies.setdefault(name, []).append("\n".join(lines[i:j + 1]))
        i += 1
        continue
    i += 1
bodies = {k: "\n".join(v) for k, v in bodies.items()}
dispatch = bodies["dispatch_call"].split("\n")
lit_re = re.compile(r'"((?:[^"\\]|\\.)*)"')
call_re = re.compile(r'\b(?:self\.)?(\w+)\(')

def arm_text(n):
    texts = []
    for idx, line in enumerate(dispatch):
        s = line.strip()
        if re.search(r'(^|\|\s*)"' + re.escape(n) + r'"(\s*\||\s*=>)', s):
            ind = len(line) - len(line.lstrip())
            k = idx
            # move to the line with => (pattern may span lines)
            while "=>" not in dispatch[k]:
                k += 1
            end = k + 1
            while end < len(dispatch):
                l2 = dispatch[end]
                ind2 = len(l2) - len(l2.lstrip())
                if ind2 <= ind and l2.strip():
                    break
                end += 1
            texts.append("\n".join(dispatch[idx:end]))
    return "\n".join(texts)

def special_text(n):
    eval_body = "\n".join(lines[440:500])
    out = []
    for m in re.finditer(r'(name == "' + re.escape(n) + r'"|"' + re.escape(n) + r'" \|[^{]*\{|\| "' + re.escape(n) + r'"\))', eval_body):
        seg = eval_body[m.start(): m.start() + 300]
        out.append(seg)
    return "\n".join(out)

def expand(text, depth):
    seen, frontier, acc = set(), [text], [text]
    for _ in range(depth):
        nxt = []
        for t in frontier:
            for c in call_re.findall(t):
                if c in bodies and c not in seen and c not in ("dispatch_call",):
                    seen.add(c)
                    nxt.append(bodies[c]); acc.append(bodies[c])
        frontier = nxt
    return "\n".join(acc)

probe_rows = {(r["package"], r["name"]): r for r in csv.DictReader(open(probe), delimiter="\t")}
with open(out, "w") as f:
    f.write("package\tname\thandled\tunhandled\n")
    for r in csv.DictReader(open(inv), delimiter="\t"):
        pr = probe_rows[(r["package"], r["name"])]
        if pr["probe"] != "recognized" or not r["formals"]:
            continue
        n = r["name"]
        text = arm_text(n) + "\n" + special_text(n)
        body = expand(text, 3)
        body = re.sub(r'data_args\(\s*args,\s*&\[[^\]]*\]\s*,?\s*\)', '', body)
        body = re.sub(r'Some\("[^"]*"\)[^\n]*=> \{\}', '', body)
        lits = set(lit_re.findall(body))
        formals = [a for a in r["formals"].split(",") if a and a != "..."]
        formals += {"rep": ["times", "length.out", "each"], "c": ["recursive", "use.names"]}.get(n, [])
        handled = [a for a in formals if a in lits]
        unhandled = [a for a in formals if a not in lits]
        f.write(f'{r["package"]}\t{n}\t{",".join(handled)}\t{",".join(unhandled)}\n')
