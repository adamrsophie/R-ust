import re
num = re.compile(r'^-?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?$')
def toks(s):
    return [t for t in re.split(r'\s+', s.replace("\\n", " ")) if t]
def same_values(a, b):
    ta, tb = toks(a), toks(b)
    if len(ta) != len(tb):
        return False
    for x, y in zip(ta, tb):
        if x == y:
            continue
        if num.match(x) and num.match(y):
            fx, fy = float(x), float(y)
            if fx == fy or abs(fx - fy) <= 1e-9 * max(abs(fx), abs(fy)):
                continue
        return False
    return True
def classify(verdict, r, u):
    if verdict in ("match", "error"):
        return verdict
    return "format" if same_values(r, u) else "value"
