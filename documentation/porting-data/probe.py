import csv, re, subprocess, sys
inv, binary, out = sys.argv[1:4]
reserved = {"if","else","repeat","while","function","for","next","break","TRUE","FALSE","NULL","Inf","NaN","NA","NA_integer_","NA_real_","NA_character_","NA_complex_","in"}
syn = re.compile(r'^(?:[A-Za-z]|\.(?![0-9]))[A-Za-z0-9._]*$')
rows = list(csv.DictReader(open(inv), delimiter="\t"))
seen = {}
with open(out, "w") as f:
    f.write("package\tname\tprobe\tdetail\n")
    for r in rows:
        n = r["name"]
        if n in reserved or not syn.match(n):
            status, detail = "nonsyntactic", ""
        else:
            key = (n, r["kind"].startswith("data"))
            if key not in seen:
                src = n if key[1] else n + "()"
                try:
                    p = subprocess.run([binary, "-e", src], capture_output=True, text=True, timeout=5)
                    msg = (p.stdout + p.stderr).strip().splitlines()
                    first = msg[0] if msg else ""
                    if f"object '{n}' not found" in first:
                        seen[key] = ("missing", "")
                    else:
                        seen[key] = ("recognized", first[:120])
                except subprocess.TimeoutExpired:
                    seen[key] = ("recognized", "timeout")
            status, detail = seen[key]
        f.write(f'{r["package"]}\t{n}\t{status}\t{detail}\n')
