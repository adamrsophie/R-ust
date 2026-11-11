import subprocess, sys
def run_both(S, binary, codes, tag):
    with open(f"{S}/{tag}_snips.tsv", "w") as f:
        f.write("code\n")
        for c in codes:
            assert "\t" not in c and "\n" not in c, c
            f.write(c + "\n")
    subprocess.run(["Rscript", "--vanilla", f"{S}/harness_r.R", f"{S}/{tag}_snips.tsv", f"{S}/{tag}_r_out.txt"], check=True, capture_output=True)
    r_out = open(f"{S}/{tag}_r_out.txt").read().splitlines()
    assert len(r_out) == len(codes), (tag, len(codes), len(r_out))
    u_out = []
    for c in codes:
        try:
            p = subprocess.run([binary, "-e", c], capture_output=True, text=True, timeout=10)
            u_out.append((p.stdout.strip() if p.returncode == 0 else "ERROR: " + p.stderr.strip()).replace("\n", "\\n"))
        except subprocess.TimeoutExpired:
            u_out.append("ERROR: timeout")
    return [r.strip() for r in r_out], u_out
