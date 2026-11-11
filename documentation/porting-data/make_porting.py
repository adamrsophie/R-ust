"""Generate documentation/PORTING.md from the probe data in this folder."""
import csv
import html
import re
import sys
from collections import Counter, defaultdict

S, OUT, COMMIT, RMATH_SURFACE = sys.argv[1:5]

PACKAGES = ["base", "stats", "utils", "methods", "graphics", "grDevices", "datasets",
            "tools", "parallel", "compiler", "grid", "splines", "stats4", "tcltk"]
RNG = {"r" + d for d in ["beta", "binom", "cauchy", "chisq", "exp", "f", "gamma", "geom", "hyper", "lnorm",
                         "logis", "nbinom", "norm", "pois", "signrank", "t", "unif", "weibull", "wilcox"]}
RNG |= {"sample", "sample.int", "set.seed"}

ROOT_CAUSES = [
    ("RC1", "Numeric comparison fixed; full comparison semantics remain partial",
     "All six operators compare numeric and logical operands numerically, retain floating point precision, "
     "and return `NA` for missing numeric operands. Character operands select string coercion and missing "
     "strings propagate `NA`. Remaining gaps include matrix attributes, factor dispatch, locale-sensitive "
     "string collation, and number-to-string conversion (RC14). "
     "Source: `compare_values` in `crates/r_ust_eval/src/lib.rs`."),
    ("RC2", "Resolved: core double NA representation",
     "Double vectors use R's NA payload, distinct from ordinary `NaN`. `NA_integer_`, `NA_real_` and "
     "`NA_character_` are parsed. Coercion, `is.nan`, printing, indexing, growth, matching, identity, "
     "and covered numeric helpers preserve the distinction. Numeric indices retain missing entries "
     "and reject mixing negative indices with positive or missing indices. Complex NA remains "
     "unsupported with the complex type. See `missing.tsv` for reference comparisons."),
    ("RC3", "Atomic vectors carry no attributes",
     "Names on atomic vectors are dropped (`c(a = 1)`, `names()`, `unlist()`, `lengths()`, `sapply()`), "
     "`table()` returns a named list, row names are renumbered, and `attr()`, `attributes()`, `structure()`, "
     "`dimnames` and arrays beyond matrices are unavailable."),
    ("RC4", "Integer values and types implemented; warnings and compact storage remain missing",
     "Valid decimal `L` literals produce integers. Colon sequences choose integer storage where R does; "
     "`seq_len` and `seq_along` return integers for ordinary lengths. Integer `+`, `-`, `*`, cumulative "
     "operations, coercion, concatenation, simplification, summaries and replacement preserve or promote "
     "types. Arithmetic overflow becomes `NA`, while an overflowing integer `sum` promotes to double. "
     "Overflow and invalid `L` qualifier warnings are not emitted. Sequences are materialized, without "
     "R's compact ALTREP storage; very large sequences remain unsupported. Other `seq` arguments remain "
     "partial. See `integers.tsv` for type and value checks."),
    ("RC5", "Random streams differ from R",
     "`set.seed()` does not reproduce R's Mersenne Twister seeding, so every random generator and `sample()` "
     "produce different draws from R even when the generator algorithm itself matches. Several generator "
     "algorithms also differ (see the Rmath section)."),
    ("RC6", "Gamma family has no `rate` argument",
     "R's signature is `dgamma(x, shape, rate = 1, scale = 1/rate)`. R-ust reads the third positional "
     "argument as `scale` and silently ignores `rate =` (RC9), so `dgamma(1.5, 2, 3)` and "
     "`dgamma(1.5, 2, rate = 3)` both differ from R. Only `scale =` matches R. Affects `dgamma`, `pgamma`, "
     "`qgamma`, `rgamma`."),
    ("RC7", "Regular expressions do not support bracket expressions",
     "Patterns such as `[aeiou]`, `[0-9]+` and `^[a-c]` do not match, so `grepl`, `grep`, `sub`, `gsub` and "
     "`strsplit` fail on them. Plain literal patterns work."),
    ("RC8", "Printing differs from R",
     "Numbers generally print with 12 decimal places (trimmed), and tiny values with 12 significant digits, "
     "instead of R's default of 7 significant digits. Vectors are neither column aligned nor "
     "formatted to a common number of decimals, long vectors do not wrap with index prefixes, "
     "`NULL` prints nothing, empty vectors print as `[1]`, data frame strings print with quotes, "
     "and non syntactic list names print without backticks."),
    ("RC9", "Builtins accept unknown named arguments",
     "`abs(-3, bogus = 2)` returns `3`; R reports an error. Closures do reject unknown arguments."),
    ("RC10", "Builtins are not first class values",
     "`f <- sum` fails with `object 'sum' not found`; builtins can only be called by name. Backtick quoted "
     "names are a parse error."),
    ("RC11", "Resolved: core operator precedence",
     "`-2^2` evaluates to `-4`, powers associate right, signed exponents work, and `!` binds below "
     "comparisons. Special operators bind above multiplication on both sides. `**` aliases `^`. "
     "Unparenthesized comparison chains are rejected. Power evaluation uses the existing `R_pow` port "
     "for R's infinite and NaN cases."),
    ("RC12", "Visibility is decided only from the top level expression",
     "Top level assignments are invisible, but `(x <- 5)` prints nothing, and a function whose last "
     "expression is an assignment prints its value. R tracks visibility through calls and parentheses."),
    ("RC13", "Polygamma accuracy",
     "`trigamma()` and `psigamma()` differ from R at about 1e-5 relative error; `digamma()` matches."),
    ("RC14", "Number to string conversion",
     "`as.character(1e6)` gives `\"1000000\"`; R gives `\"1e+06\"` (15 significant digits with R's "
     "scientific notation rules)."),
    ("RC15", "No output from inside loops or functions",
     "`print()` returns its argument without printing, and `cat()`, `message()` and `writeLines()` are "
     "missing. Only values of top level expressions reach the console, so `for (i in 1:2) print(i)` prints "
     "nothing."),
]

FN_NOTES = {
    "rep": "`each` is silently ignored (RC9).",
    "sapply": "`USE.NAMES` is accepted but ignored; result names are dropped (RC3).",
    "table": "Returns a named list, not a `table` object (RC3).",
    "subset": "Row names are renumbered (RC3).",
    "na.omit": "The `na.action` attribute and `omit` class are not attached (RC3).",
    "data.frame": "Character columns print with quotes (RC8).",
    "split": "Group names print as `$1` instead of R's backtick form (RC8).",
    "names": "Returns nothing for named atomic vectors (RC3).",
    "c": "Element names are dropped (RC3).",
    "unlist": "Names are dropped (RC3).",
    "lengths": "Names are dropped (RC3).",
    "as.character": "Large numbers convert without scientific notation (RC14).",
    "trigamma": "Accuracy about 1e-5 (RC13).",
    "psigamma": "Accuracy about 1e-5 (RC13).",
    "print": "Returns its argument without printing, so it shows nothing inside loops or functions (RC15); formatting options are ignored (RC8).",
    "grep": "Bracket expressions fail (RC7); no match prints `[1]` (RC8).",
    "grepl": "Bracket expressions fail (RC7).",
    "sub": "Bracket expressions fail (RC7).",
    "gsub": "Bracket expressions fail (RC7).",
    "strsplit": "Bracket expressions fail (RC7).",
}
for g in ["dgamma", "pgamma", "qgamma", "rgamma"]:
    FN_NOTES[g] = "No `rate` argument; the third positional argument is read as `scale` (RC6)."

OPS = {
    "+": ("Partial", "Integer values and overflow are implemented; warnings are missing (RC4)."),
    "-": ("Partial", "Integer values and overflow are implemented; warnings are missing (RC4)."),
    "*": ("Partial", "Integer values and overflow are implemented; warnings are missing (RC4)."), "/": ("Done", ""),
    "^": ("Done", "Core precedence fixed (RC11)."),
    "<": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."), "<=": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."),
    ">": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."), ">=": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."),
    "==": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."), "!=": ("Partial", "Numeric comparison fixed; broader gaps remain (RC1)."), "!": ("Done", ""), "&": ("Done", ""), "&&": ("Done", ""),
    "|": ("Done", ""), "||": ("Done", ""), "{": ("Done", ""),
    "(": ("Partial", "A parenthesized assignment does not print (RC12)."),
    ":": ("Partial", "Integer storage implemented; compact long-vector storage is missing (RC4)."),
    "<-": ("Done", ""), "=": ("Done", ""),
    "<<-": ("Partial", "Assigns in the global environment instead of the nearest enclosing function environment."),
    "if": ("Done", ""), "for": ("Done", ""), "while": ("Done", ""), "repeat": ("Done", ""),
    "break": ("Done", ""), "next": ("Done", ""),
    "function": ("Partial", "No reference semantics for enclosing environments; printing shows only the header."),
    "%in%": ("Done", ""),
    "[": ("Partial", "Character indexing only for lists; `drop` and empty indices with named arguments are parse errors."),
    "[[": ("Partial", "No recursive indexing with a vector index."),
    "$": ("Partial", "No partial name matching."),
    "[<-": ("Partial", "Growing a double vector pads with `NA` (RC2); general replacement semantics remain incomplete."),
    "[[<-": ("Done", ""), "$<-": ("Done", ""),
    "names<-": ("Partial", "Works for lists and data frames only (RC3)."),
    "[.data.frame": ("Partial", "Row and column indexing work; row names are renumbered (RC3)."),
    "[[.data.frame": ("Done", ""), "$<-.data.frame": ("Done", ""), "[[<-.data.frame": ("Done", ""),
    "[<-.data.frame": ("Done", ""), "[.factor": ("Done", ""),
}

LANG_NOTES = {
    "Integer literal (1L)": "Integer storage implemented; invalid L qualifiers do not warn (RC4).",
    "Hexadecimal literal": "Parse error.",
    "Complex literal": "Parse error; no complex type.",
    "Unicode escape": "`\\u` escapes are not decoded.",
    "Raw string literal": "Parse error.",
    "Typed NA constants": "Atomic integer, double and character NA constants are implemented. Complex NA remains missing with the complex type.",
    "Backtick-quoted names": "Parse error (RC10).",
    "Built-in constants (pi, letters, LETTERS, month.name)": "None of these constants are defined.",
    "print() inside a loop": "RC15.",
    "print() inside a function": "RC15.",
    "Integer overflow to NA": "Value matches; overflow warnings are missing (RC4).",
    "Character (name) index": "Works for lists, not atomic vectors (RC3).",
    "Partial matching with $": "Returns `NULL`.",
    "Recursive [[ ]] indexing": "Returns the first level only.",
    "drop = FALSE": "Empty index followed by a named argument is a parse error.",
    "Superassignment <<-": "Assignment works; the function call result also prints (RC12).",
    "Printing via parentheses": "RC12.",
    "names<- replacement": "Atomic vectors cannot hold names (RC3).",
    "User-defined replacement function": "Defining `name<-` needs a quoted or backtick name, which does not parse (RC10).",
    "User-defined infix %op%": "Defining `%op%` needs a quoted or backtick name, which does not parse; `%op%` calls are not implemented.",
    "Closures capture environment": "`<<-` inside a closure does not update the enclosing function environment.",
    "Functions as values": "RC10.",
    "Function printing": "Prints only the header line.",
    "Partial argument matching": "Only exact names and positions match.",
    "Unknown named argument to a builtin is an error": "RC9.",
    "Matrix with dimnames": "`dimnames` is ignored (RC3).",
    "Default 7 significant digits": "RC8.",
    "Long vector wraps with index prefix": "RC8.",
    "Named vector printing": "RC3 and RC8.",
    "Nested list printing": "Nested element prefixes such as `$b$c` are not printed.",
    "Data frame printing": "Strings print with quotes (RC8).",
    "Table printing": "Prints a named list (RC3).",
    "print(quote = FALSE)": "`quote` is ignored.",
    "NULL printing": "Prints nothing (RC8).",
}


def read(name):
    with open(f"{S}/{name}") as f:
        return list(csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE))


def code(name):
    return "`" + name.replace("|", "\\|") + "`"


def ev(text, limit=90):
    text = text.replace("\\n", " ⏎ ")
    if len(text) > limit:
        text = text[:limit].rstrip() + " …"
    return "<code>" + html.escape(text).replace("|", "&#124;") + "</code>"


inventory = read("r_inventory.tsv")
probe = {(r["package"], r["name"]): r["probe"] for r in read("probe.tsv")}
formals = {(r["package"], r["name"]): r for r in read("formals.tsv")}
fn_rows = defaultdict(list)
for r in read("fn_diff.tsv"):
    fn_rows[r["name"]].append(r)
lang = read("lang.tsv")
operators = read("operators.tsv")
missing_values = read("missing.tsv")
integers = read("integers.tsv")


def fn_status(pkg, name, kind):
    if probe[(pkg, name)] == "missing":
        return "Missing", "", ""
    if probe[(pkg, name)] == "nonsyntactic":
        status, note = OPS.get(name, ("Missing", ""))
        return status, note, ""
    rows = fn_rows.get(name, [])
    classes = {r["class"] for r in rows}
    f = formals.get((pkg, name))
    unref = []
    if f:
        unref = [a for a in f["unhandled"].split(",") if a]
        allf = [a for a in inventory_formals[(pkg, name)] if a and a != "..."]
        if allf and allf[0] in unref:
            unref.remove(allf[0])
    notes = []
    evidence = ""
    bad = [r for r in rows if r["class"] in ("value", "error")]
    if name in RNG:
        notes.append("Draws differ from R (RC5).")
    if name in FN_NOTES:
        notes.append(FN_NOTES[name])
    if bad:
        b = bad[0]
        evidence = f"{ev(b['call'], 60)}: R {ev(b['r_out'], 60)}, R-ust {ev(b['rust_out'], 60)}"
    elif "format" in classes and name not in FN_NOTES:
        notes.append("Values match; printing differs (RC8).")
    status = "Partial" if (bad or unref or name in RNG or name in FN_NOTES) else "Done"
    if unref:
        notes.insert(0, "Arguments not referenced: " + ", ".join(code(a) for a in unref) + ".")
    return status, " ".join(notes), evidence


inventory_formals = {(r["package"], r["name"]): r["formals"].split(",") for r in inventory}

by_pkg = defaultdict(list)
for r in inventory:
    by_pkg[r["package"]].append(r)

results = {}
for pkg in PACKAGES:
    for r in by_pkg[pkg]:
        results[(pkg, r["name"])] = fn_status(pkg, r["name"], r["kind"])

lang_status = []
for r in lang:
    k = r["class"]
    if k == "match":
        st = "Done"
    elif k == "format":
        st = "Partial" if r["category"] == "Printing" else "Done"
    elif k == "value":
        st = "Partial"
    else:
        st = "Missing"
    if r["feature"] == "Integer overflow to NA" and st == "Done":
        st = "Partial"
    lang_status.append((r, st))

out = []
w = out.append
w("# R-ust porting status")
w("")
w(f"Reference: R 4.6.0 (2026-04-24). R-ust commit `{COMMIT}`. Generated 2026-11-11.")
w("")
w("This file records, for every object exported by the R 4.6.0 base distribution and for the core language "
  "features, whether R-ust implements it, implements it in part, or lacks it. The R development trunk "
  "(revision r90875, R 4.7.0 under development) is checked out in `documentation/R-devel` and has not been "
  "compared yet.")
w("")
w("## Contents")
w("")
w("1. [Summary](#summary)")
w("2. [Method](#method)")
w("3. [Root causes](#root-causes)")
w("4. [Language features](#language-features)")
w("5. [Runtime and infrastructure](#runtime-and-infrastructure)")
w("6. [Rmath C library](#rmath-c-library)")
w("7. [Functions by package](#functions-by-package)")
w("8. [Names in R-ust that R 4.6.0 does not have](#names-in-r-ust-that-r-460-does-not-have)")
w("")
w("## Summary")
w("")
w("| Package | Objects | Done | Partial | Missing |")
w("| --- | ---: | ---: | ---: | ---: |")
tot = Counter()
for pkg in PACKAGES:
    c = Counter(results[(pkg, r["name"])][0] for r in by_pkg[pkg])
    n = len(by_pkg[pkg])
    tot.update(c); tot["n"] += n
    w(f"| {pkg} | {n} | {c['Done']} | {c['Partial']} | {c['Missing']} |")
w(f"| **Total** | **{tot['n']}** | **{tot['Done']}** | **{tot['Partial']}** | **{tot['Missing']}** |")
w("")
lc = Counter(st for _, st in lang_status)
w(f"Language features: {len(lang_status)} checked, {lc['Done']} done, {lc['Partial']} partial, {lc['Missing']} missing.")
w("")
w("Recommended packages shipped with R (MASS, lattice, Matrix, nlme, survival, boot, cluster, codetools, foreign, "
  "KernSmooth, rpart, class, nnet, spatial, mgcv) are not counted above; none of them can run on R-ust yet.")
w("")
w("## Method")
w("")
w("1. **Inventory.** Every object in the base environment and every export of the other 13 base distribution "
  "packages was listed from an installed R 4.6.0, with its formal arguments. For S3 generics the formals of the "
  "default method were added, and for `rep` and `c` the documented arguments that live in `...`.")
w("2. **Availability.** Each syntactic name was evaluated in R-ust (`name()` for functions, `name` for data). "
  "`object 'name' not found` means Missing. Operators, replacement functions and other non syntactic names were "
  "classified by hand from the language feature checks below.")
w("3. **Arguments.** For each available function, the Rust code that implements it (its dispatch arm and the "
  "functions it calls, three levels deep) was scanned for string literals naming R's formal arguments. A formal "
  "that never appears is listed as *not referenced*; that result is reliable, and because unknown named "
  "arguments are silently dropped (RC9), passing such an argument by name has no effect. A formal that does appear is only "
  "*referenced*, which does not prove it is honored (for example `rep(each =)` and `sapply(USE.NAMES =)` are "
  "referenced or accepted but ignored). The first formal counts as referenced because every function accepts "
  "it by position.")
w("4. **Behavior.** One to three representative calls per available function, including mixed width numbers "
  "such as `c(10, 9, 100, 2)` for ordering functions, were run in R 4.6.0 with `options(digits = 12)` and in "
  "R-ust, and the printed results were compared. Identical output is a match. Output that differs only in "
  "layout, or in numbers equal to a relative tolerance of 1e-9, is a formatting difference. Anything else is a "
  "value difference.")
w(f"5. **Language features.** {len(lang_status)} snippets covering syntax, operators, indexing, assignment, "
  "control flow, functions, environments, conditions, object systems, types and printing were run the same way.")
w(f"6. **Operator combinations.** {len(operators)} additional expressions combine supported binary operators "
  "and unary signs/negation. `check_operators.py` requires matching values under the same printed numeric "
  "tolerance, or an error in both runtimes; error message wording is not compared. Inputs and results are "
  "recorded in `operators_snips.tsv`, `operators_r_out.txt`, and `operators.tsv`. The reference harness "
  "escapes embedded newlines and checks case counts before pairing results.")
w(f"7. **Missing values.** {len(missing_values)} expressions in `check_missing.py` compare NA and NaN "
  "across arithmetic, summaries, unary math, coercion, matching, indexing and matrix summaries. "
  "`missing_snips.tsv`, `missing_r_out.txt`, and `missing.tsv` preserve inputs and results. "
  "This is a targeted regression corpus, not full-range numerical validation.")
w(f"8. **Integer types and values.** {len(integers)} expressions in `check_integers.py` compare "
  "`typeof` and values for mixed arithmetic, boundary literals, sequences, coercion, summaries, "
  "simplification and replacement. Inputs and outputs are in `integers_snips.tsv`, "
  "`integers_r_out.txt`, and `integers.tsv`. Warning messages and compact storage are not checked.")
w("")
w("Status words:")
w("")
w("- **Done**: available, no formal argument is unreferenced, and every reference call returned the same values as "
  "R. Formatting differences alone do not prevent Done, because printing is tracked as its own feature (RC8). "
  "Done rests on a small number of calls, not on exhaustive testing.")
w("- **Partial**: available, but with unreferenced arguments, a value difference, an error, or a known root cause.")
w("- **Missing**: not available in R-ust.")
w("- **Unverified** (Rmath only): implemented in Rust but with no R level reference call in this run.")
w("")
w("The scripts and raw results that produced this file are in `documentation/porting-data/`.")
w("")
w("## Root causes")
w("")
w("Most partial results trace back to a few shared defects. Rows below cite these identifiers.")
w("")
for rc, title, body in ROOT_CAUSES:
    w(f"- **{rc}. {title}.** {body}")
w("")

w("## Language features")
w("")
cats = []
for r, _ in lang_status:
    if r["category"] not in cats:
        cats.append(r["category"])
for cat in cats:
    rows = [(r, st) for r, st in lang_status if r["category"] == cat]
    c = Counter(st for _, st in rows)
    w(f"### {cat}")
    w("")
    w(f"{len(rows)} checked: {c['Done']} done, {c['Partial']} partial, {c['Missing']} missing.")
    w("")
    w("| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |")
    w("| --- | --- | --- | --- | --- | --- |")
    for r, st in rows:
        note = LANG_NOTES.get(r["feature"], "")
        if not note and r["class"] == "format" and cat != "Printing":
            note = "Values match; printing differs (RC8)."
        w(f"| {html.escape(r['feature']).replace('|', '&#124;')} | {st} | {ev(r['code'], 70)} | {ev(r['r_out'], 50)} | {ev(r['rust_out'], 50)} | {note} |")
    w("")

w("## Runtime and infrastructure")
w("")
w("| Component | Status | Notes |")
w("| --- | --- | --- |")
INFRA = [
    ("Parser", "Partial", "Handles the constructs marked Done above. Missing: hex, complex and raw string literals, backticks, `::`, `:::`, `@`, `~`, `|>`, `\\(x)`, `%op%` operators other than `%in%`, empty arguments, quoted assignment targets, and invalid L qualifier warnings."),
    ("Evaluator", "Partial", "Lazy promises, closures, `...`, exact and positional matching, and control flow work. Missing: environments as objects, reference semantics for enclosing environments, S3, S4 and reference class dispatch, condition system, `on.exit`, `sys.*` frame access, bytecode."),
    ("Object model (`r_ust_sexp`)", "Partial", "SEXPTYPE codes match R 4.6.0. Runtime values: NULL, logical, integer, double, character, list, factor, data frame, matrix, closure, promise. Missing: complex, raw, environment, symbol and language objects at R level, expression vectors, S4 objects, external pointers, weak references, general attributes."),
    ("Memory management (`r_ust_gc`)", "Partial", "A handle model and protect stack exist; evaluator values are plain Rust values with no R heap, generational collector, `gc()`, or memory profiling."),
    ("Printing", "Partial", "See RC8."),
    ("Interactive console (REPL)", "Missing", "`r_ust_cli` with no expression or file reports that the console is not implemented."),
    ("Command line front end", "Partial", "`r_ust_cli` accepts `-e`, `-f`/`--file`, a script path, `-q`/`--quiet`, `--vanilla` (no effect), `--version` and `--args` (stops option parsing). Missing: `Rscript`, `R CMD`, `--no-echo`, `--save`/`--no-save`, `--restore`, `commandArgs()`, other R options."),
    ("Startup files and environment", "Missing", "`.Rprofile`, `Rprofile.site`, `.Renviron`, `R_LIBS` and site library handling."),
    ("`options()` and `getOption()`", "Missing", ""),
    ("Locale, encodings, `Encoding()`", "Missing", "Strings are Rust UTF-8 strings; `\\u` escapes are not decoded."),
    ("Condition system", "Missing", "`stop`, `warning`, `message`, `tryCatch`, `try`, `withCallingHandlers`, restarts. Errors abort evaluation with a message."),
    ("S3 dispatch", "Missing", "`UseMethod`, `NextMethod`, group generics, `class<-`, `inherits`, `structure`."),
    ("S4 (`methods`)", "Missing", "`setClass`, `new`, `setGeneric`, `setMethod`, `@`, `isVirtualClass`, `show`."),
    ("Reference classes", "Missing", "`setRefClass`."),
    ("Bytecode compiler (`compiler`)", "Missing", ""),
    ("Package system", "Missing", "`library`, `require`, namespaces, `loadNamespace`, `::`, lazy loading, `install.packages`, `R CMD INSTALL`/`check`/`build`."),
    ("Serialization", "Missing", "`saveRDS`, `readRDS`, `save`, `load`, `serialize`, workspace images."),
    ("Connections and file I/O", "Missing", "`file`, `url`, `gzfile`, `readLines`, `writeLines`, `scan`, `read.table`, `sink`, `source`."),
    ("System interface", "Missing", "`Sys.time`, `Sys.getenv`, `system`, `file.exists`, `proc.time`, `Sys.sleep`."),
    ("Dates and times", "Missing", "`Date`, `POSIXct`, `POSIXlt`, `difftime`, `strptime`, `format`."),
    ("C API (`Rinternals.h`, `.Call`, `.C`, `.External`)", "Missing", "No R object API for compiled code; packages with compiled code cannot load."),
    ("Rmath standalone C API", "Partial", "All 154 symbols exported from `r_ust_ffi`; accuracy caveats in the next section."),
    ("Graphics engine and devices", "Missing", "`graphics`, `grDevices`, `grid`, PDF, PNG, screen devices."),
    ("Tcl/Tk (`tcltk`)", "Missing", ""),
    ("Parallel (`parallel`)", "Missing", ""),
    ("Help system", "Missing", "`help`, `?`, Rd rendering, `example`."),
    ("Debugging", "Missing", "`browser`, `debug`, `traceback`, `trace`, `Rprof`."),
    ("Datasets (`datasets`)", "Missing", "No data objects are defined."),
]
for comp, st, note in INFRA:
    w(f"| {comp} | {st} | {note.replace('|', chr(92) + '|')} |")
w("")

w("## Rmath C library")
w("")
w("`r_ust_nmath` implements and `r_ust_ffi` exports all 154 public functions of R's standalone `Rmath.h`. "
  "The evidence below comes from R level calls; no C level comparison against R's own `libRmath` was run.")
w("")
rm_names = re.findall(r'"([^"]+)"', re.search(r'pub const PUBLIC_FUNCTIONS[^=]*=\s*&\[(.*?)\];',
                                              open(RMATH_SURFACE).read(), re.S).group(1))
APPROX = {}
for n in ["bessel_i", "bessel_i_ex", "bessel_j", "bessel_j_ex", "bessel_k", "bessel_k_ex", "bessel_y", "bessel_y_ex"]:
    APPROX[n] = "Series, reflection and asymptotic approximations instead of R's Cody and Amos routines."
for n in ["dnbeta", "pnbeta", "qnbeta", "dnf", "pnf", "qnf", "dnt", "pnt", "qnt", "dnchisq", "pnchisq", "qnchisq"]:
    APPROX[n] = "Mixture, integration and bisection methods; tail agreement with R not validated."
for n in ["ptukey", "qtukey"]:
    APPROX[n] = "Monotone approximation instead of R's quadrature."
for n in ["rgamma", "rbeta", "rf", "rt", "rbinom", "rpois", "rhyper", "rsignrank", "rwilcox", "rmultinom",
          "rchisq", "rnbinom", "rnbinom_mu", "rnchisq"]:
    APPROX[n] = "Generator algorithm differs from R; draws are not R's stream."
for n in ["trigamma", "psigamma", "tetragamma", "pentagamma", "dpsifn"]:
    APPROX[n] = "Polygamma accuracy about 1e-5 where checked (RC13)."
RLEVEL = {"gammafn": "gamma", "lgammafn": "lgamma", "beta": "beta", "lbeta": "lbeta", "choose": "choose",
          "lchoose": "lchoose", "digamma": "digamma"}
rm_rows = []
for n in rm_names:
    if n in APPROX:
        rm_rows.append((n, "Partial", APPROX[n]))
        continue
    rname = RLEVEL.get(n, n)
    rows = fn_rows.get(rname, [])
    if n in ("signrank_free", "wilcox_free"):
        rm_rows.append((n, "Done", "No-op; exact tables need no cache."))
    elif rows and all(r["class"] in ("match", "format") for r in rows) and rname not in RNG:
        rm_rows.append((n, "Done", f"Matches R through `{rname}()`."))
    elif rname in ("dgamma", "pgamma", "qgamma"):
        rm_rows.append((n, "Done", "Matches R when called with `scale`; the R level wrapper has RC6."))
    elif rname in RNG:
        rm_rows.append((n, "Unverified", "Stream comparison blocked by RC5."))
    else:
        rm_rows.append((n, "Unverified", ""))
rc = Counter(st for _, st, _ in rm_rows)
w(f"{len(rm_rows)} functions: {rc['Done']} done, {rc['Partial']} partial, {rc['Unverified']} unverified.")
w("")
w("| Function | Status | Notes |")
w("| --- | --- | --- |")
for n, st, note in rm_rows:
    w(f"| {code(n)} | {st} | {note} |")
w("")

w("## Functions by package")
w("")
w("Each package lists the available objects with their status and evidence, then the missing objects. "
  "Evidence shows the first call whose values differ from R.")
w("")
for pkg in PACKAGES:
    rows = by_pkg[pkg]
    c = Counter(results[(pkg, r["name"])][0] for r in rows)
    w(f"### {pkg}")
    w("")
    w(f"{len(rows)} objects: {c['Done']} done, {c['Partial']} partial, {c['Missing']} missing.")
    w("")
    avail = [r for r in rows if results[(pkg, r["name"])][0] != "Missing"]
    if avail:
        w("| Name | Kind | Status | Notes | Evidence |")
        w("| --- | --- | --- | --- | --- |")
        for r in avail:
            st, note, evid = results[(pkg, r["name"])]
            kind = r["kind"].replace("data:", "")
            w(f"| {code(r['name'])} | {kind} | {st} | {note} | {evid} |")
        w("")
    missing = [r["name"] for r in rows if results[(pkg, r["name"])][0] == "Missing"]
    meta = [m for m in missing if m.startswith(".__")]
    missing = [m for m in missing if not m.startswith(".__")]
    if missing:
        w(f"Missing ({len(missing)}):")
        w("")
        cols = 4
        w("| " + " | ".join([" "] * cols) + " |")
        w("| " + " | ".join(["---"] * cols) + " |")
        for i in range(0, len(missing), cols):
            chunk = [code(m) for m in missing[i:i + cols]]
            chunk += [""] * (cols - len(chunk))
            w("| " + " | ".join(chunk) + " |")
        w("")
    if meta:
        w(f"S4 class and generic metadata objects, missing with the S4 system ({len(meta)}): "
          + ", ".join(code(m) for m in meta) + ".")
        w("")

w("## Names in R-ust that R 4.6.0 does not have")
w("")
w("| Name | Notes |")
w("| --- | --- |")
w("| `as.real` | Removed from R in 3.0.0; R-ust treats it as `as.double`. |")
w("| `is.real` | Removed from R in 3.0.0; R-ust treats it as `is.double`. |")
w("| `rnchisq` | Not an R function; R uses `rchisq(n, df, ncp)`. Exposed from the Rmath API. |")
w("| `set_seed` | Not an R function; alias of `set.seed`. |")
w("")
open(OUT, "w").write("\n".join(out))
print("rows:", len(out), dict(tot))
