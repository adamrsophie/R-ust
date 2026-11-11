# One representative call per R-ust builtin: name -> R expression
C = {}
for f in ["abs","sqrt","exp","expm1","log","log10","log2","log1p","sin","cos","tan","asin","acos","atan","sinh","cosh","tanh","asinh","atanh","floor","ceiling","trunc","sign","gamma","lgamma","digamma","trigamma","factorial","lfactorial","cumsum","cumprod","cummax","cummin"]:
    C[f] = f"{f}(c(0.5, -0.25, 0.75))" if f not in ("sqrt","log","log10","log2","gamma","lgamma","digamma","trigamma","factorial","lfactorial","asin","acos","atanh") else f"{f}(c(0.5, 0.25, 0.75))"
C["acosh"] = "acosh(c(1.5, 2, 3))"
C["round"] = "round(c(2.567, -1.005, 12345.6789), 2)"
C["signif"] = "signif(c(2.567, -1.005, 12345.6789), 3)"
C["logb"] = "logb(c(8, 100), 2)"
C["atan2"] = "atan2(c(1, -1), c(2, 2))"
C["beta"] = "beta(2, 3)"; C["lbeta"] = "lbeta(2, 3)"
C["choose"] = "choose(5, 0:5)"; C["lchoose"] = "lchoose(5, 2)"
C["psigamma"] = "psigamma(c(1.5, 2), 2)"
for f in ["sum","prod","max","min","range","mean","median","var","sd"]:
    C[f] = f"{f}(c(4, 1, NA, 7), na.rm = TRUE)"
C["all"] = "all(c(TRUE, NA, FALSE))"; C["any"] = "any(c(FALSE, NA, TRUE))"
C["pmax"] = "pmax(c(1, 5, 3), c(4, 2, 6))"; C["pmin"] = "pmin(c(1, 5, 3), c(4, 2, 6))"
C["xor"] = "xor(c(TRUE, FALSE), c(TRUE, TRUE))"
C["isTRUE"] = "isTRUE(c(TRUE, TRUE))"; C["isFALSE"] = "isFALSE(FALSE)"
C["identical"] = "identical(c(a = 1), c(a = 1))"
for f in ["is.na","is.nan","is.finite","is.infinite"]:
    C[f] = f"{f}(c(1, NA, NaN, Inf))"
for f in ["is.null","is.atomic","is.character","is.double","is.integer","is.list","is.logical","is.numeric","is.function","is.vector","is.matrix","is.data.frame","is.factor","is.ordered"]:
    C[f] = f"{f}(1:3)"
C["is.unsorted"] = "is.unsorted(c(1, 3, 2))"
C["as.numeric"] = 'as.numeric(c("1.5", "2", "1e3"))'; C["as.double"] = 'as.double(c("1.5", "2"))'
C["as.integer"] = 'as.integer(c(1.9, -2.7, 3))'; C["as.logical"] = 'as.logical(c("TRUE", "false", "T", "yes"))'
C["as.character"] = 'as.character(c(1.5, 2, 1e6, 123456789))'
C["as.list"] = "as.list(1:2)"; C["as.factor"] = 'as.factor(c("b", "a", "b"))'
C["unlist"] = "unlist(list(a = 1, b = list(c = 2, d = 3)))"
C["c"] = 'c(a = 1, b = 2, 3)'; C["list"] = 'list(1, "a", TRUE)'
C["length"] = "length(1:10)"; C["lengths"] = "lengths(list(a = 1:3, b = 1))"
C["names"] = "names(c(a = 1, b = 2))"; C["class"] = "class(matrix(1:4, 2))"
C["typeof"] = "typeof(1L)"; C["mode"] = "mode(1L)"
C["seq"] = "seq(0, 1, by = 0.25)"; C["seq.int"] = "seq.int(10, 1, by = -3)"
C["seq_len"] = "seq_len(4)"; C["seq_along"] = 'seq_along(c("a", "b", "c"))'
C["rep"] = "rep(1:2, times = 3, each = 2)"; C["rep.int"] = "rep.int(1:2, 2)"; C["rep_len"] = "rep_len(1:3, 7)"
C["append"] = "append(1:3, 99, after = 1)"; C["replace"] = "replace(1:5, 2, 0)"
C["ifelse"] = "ifelse(c(1, -1, 0) > 0, \"pos\", \"nonpos\")"
C["rev"] = "rev(c(3, 1, 2))"; C["sort"] = "sort(c(3, 1, NA, 2), decreasing = TRUE)"
C["order"] = "order(c(3, 1, 2, 1))"; C["rank"] = "rank(c(10, 20, 10, 30))"
C["diff"] = "diff(c(1, 4, 9, 16), lag = 2)"
C["head"] = "head(letters, 3)" ; C["tail"] = "tail(1:10, 3)"
C["which"] = "which(c(FALSE, TRUE, NA, TRUE))"; C["which.max"] = "which.max(c(3, 9, 2))"; C["which.min"] = "which.min(c(3, 9, 2))"
C["match"] = "match(c(\"b\", \"z\"), c(\"a\", \"b\"))"
C["unique"] = "unique(c(3, 1, 3, 2, 1))"; C["duplicated"] = "duplicated(c(3, 1, 3, 2, 1))"
C["anyDuplicated"] = "anyDuplicated(c(3, 1, 3))"
C["union"] = "union(c(1, 2), c(2, 3))"; C["intersect"] = "intersect(1:5, 3:8)"
C["setdiff"] = "setdiff(1:5, 2:3)"; C["setequal"] = "setequal(1:3, 3:1)"
C["split"] = 'split(1:6, c("a", "b"))'; C["tabulate"] = "tabulate(c(1, 2, 2, 5))"
C["table"] = 'table(c("a", "b", "a", "c"))'
C["factor"] = 'factor(c("lo", "hi", "lo"), levels = c("lo", "hi"))'
C["ordered"] = 'ordered(c("lo", "hi"), levels = c("lo", "hi"))'
C["levels"] = 'levels(factor(c("b", "a")))'; C["nlevels"] = 'nlevels(factor(c("b", "a")))'
C["paste"] = 'paste("a", 1:3, sep = "-")'; C["paste0"] = 'paste0("x", 1:3, collapse = "+")'
C["nchar"] = 'nchar(c("abc", "", NA))'; C["nzchar"] = 'nzchar(c("abc", ""))'
C["toupper"] = 'toupper("abc")'; C["tolower"] = 'tolower("ABC")'
C["chartr"] = 'chartr("abc", "xyz", "aabbcc")'
C["substr"] = 'substr("abcdef", 2, 4)'; C["substring"] = 'substring("abcdef", 1:3, 3:5)'
C["strtrim"] = 'strtrim("abcdef", 3)'; C["strrep"] = 'strrep("ab", 3)'
C["startsWith"] = 'startsWith(c("apple", "banana"), "a")'; C["endsWith"] = 'endsWith(c("apple", "banana"), "a")'
C["grepl"] = 'grepl("an", c("banana", "apple"))'; C["grep"] = 'grep("a", c("abc", "xyz", "cba"))'
C["sub"] = 'sub("a", "A", "banana")'; C["gsub"] = 'gsub("[aeiou]", "", "banana")'
C["strsplit"] = 'strsplit("a,b,c", ",")'; C["trimws"] = 'trimws("  hi  ")'
C["matrix"] = "matrix(1:6, nrow = 2)"; C["dim"] = "dim(matrix(1:6, 2))"
C["nrow"] = "nrow(matrix(1:6, 2))"; C["ncol"] = "ncol(matrix(1:6, 2))"
C["t"] = "t(matrix(1:6, 2))"; C["diag"] = "diag(2)"
C["row"] = "row(matrix(1:4, 2))"; C["col"] = "col(matrix(1:4, 2))"
C["lower.tri"] = "lower.tri(matrix(1:9, 3))"; C["upper.tri"] = "upper.tri(matrix(1:9, 3))"
for f in ["rowSums","colSums","rowMeans","colMeans"]:
    C[f] = f"{f}(matrix(1:6, 2))"
C["crossprod"] = "crossprod(matrix(1:4, 2))"; C["tcrossprod"] = "tcrossprod(matrix(1:4, 2))"
C["cbind"] = "cbind(1:2, 3:4)"; C["rbind"] = "rbind(1:2, 3:4)"
C["apply"] = "apply(matrix(1:6, 2), 1, sum)"
C["lapply"] = "lapply(1:2, function(i) i * 10)"; C["sapply"] = "sapply(c(a = 1, b = 2), function(i) i^2)"
C["data.frame"] = 'data.frame(x = 1:2, y = c("a", "b"))'
C["with"] = "with(data.frame(x = 1:3), sum(x))"
C["subset"] = "subset(data.frame(x = 1:4, y = 4:1), x > 2)"
C["transform"] = "transform(data.frame(x = 1:2), y = x * 2)"
C["complete.cases"] = "complete.cases(data.frame(x = c(1, NA, 3)))"
C["na.omit"] = "na.omit(c(1, NA, 3))"
C["sample"] = "set.seed(42); sample(10, 4)"; C["sample.int"] = "set.seed(42); sample.int(100, 3)"
C["set.seed"] = "set.seed(1); runif(2)"
C["print"] = "print(c(1.5, 2))"
C["missing"] = "f <- function(a) missing(a); f()"
C["return"] = "f <- function() { return(5); 6 }; f()"
C["T"] = "T"; C["F"] = "F"
D = {"norm": ("c(-1, 0, 1.5)", "1.96", "0.975", ""), "unif": ("0.3, 0, 2", "0.3, 0, 2", "0.3, 0, 2", "0, 2"),
     "binom": ("3, 10, 0.3", "3, 10, 0.3", "0.5, 10, 0.3", "10, 0.3"), "pois": ("2, 3", "2, 3", "0.5, 3", "3"),
     "geom": ("2, 0.3", "2, 0.3", "0.5, 0.3", "0.3"), "nbinom": ("2, 5, 0.5", "2, 5, 0.5", "0.5, 5, 0.5", "5, 0.5"),
     "hyper": ("1, 5, 10, 4", "1, 5, 10, 4", "0.5, 5, 10, 4", "5, 10, 4"), "exp": ("1, 2", "1, 2", "0.5, 2", "2"),
     "gamma": ("1.5, 2, 3", "1.5, 2, 3", "0.5, 2, 3", "2, 3"), "beta": ("0.3, 2, 5", "0.3, 2, 5", "0.5, 2, 5", "2, 5"),
     "lnorm": ("1.5", "1.5", "0.5", ""), "weibull": ("1.5, 2, 1", "1.5, 2, 1", "0.5, 2, 1", "2, 1"),
     "logis": ("0.5", "0.5", "0.7", ""), "cauchy": ("0.5", "0.5", "0.7", ""), "chisq": ("2, 3", "2, 3", "0.95, 3", "3"),
     "t": ("1.5, 5", "1.5, 5", "0.975, 5", "5"), "f": ("1.5, 3, 10", "1.5, 3, 10", "0.95, 3, 10", "3, 10"),
     "signrank": ("5, 10", "5, 10", "0.5, 10", "10"), "wilcox": ("5, 4, 6", "5, 4, 6", "0.5, 4, 6", "4, 6")}
for d, (da, pa, qa, ra) in D.items():
    C["d" + d] = f"d{d}({da})"; C["p" + d] = f"p{d}({pa})"; C["q" + d] = f"q{d}({qa})"
    C["r" + d] = f"set.seed(42); r{d}(3{', ' + ra if ra else ''})"

# Normalize to lists and add stress calls (mixed-width numbers expose text ordering).
C = {k: [v] for k, v in C.items()}
for d, (da, pa, qa, ra) in D.items():
    C["d" + d].append(f"d{d}({da}, log = TRUE)")
    C["p" + d].append(f"p{d}({pa}, lower.tail = FALSE)")
    C["q" + d].append(f"q{d}({qa}, lower.tail = FALSE)")
W = "c(10, 9, 100, 2)"
for f in ["sort", "order", "rank", "rev", "max", "min", "range", "which.max", "which.min", "cummax", "cummin",
          "median", "unique", "duplicated", "anyDuplicated", "is.unsorted", "tabulate"]:
    C[f].append(f"{f}({W})")
C["pmax"].append(f"pmax({W}, c(3, 30, 3, 30))"); C["pmin"].append(f"pmin({W}, c(3, 30, 3, 30))")
C["match"].append(f"match(c(100, 2), {W})")
for f in ["union", "intersect", "setdiff"]:
    C[f].append(f"{f}({W}, c(2, 20))")
C["setequal"].append(f"setequal({W}, c(2, 9, 10, 100))")
C["table"].append(f"table({W})")
C["split"].append(f"split({W}, c(1, 2, 1, 2))")
C["is.integer"] = ["is.integer(as.integer(c(1, 2)))"]
C["is.double"] = ["is.double(as.integer(c(1, 2)))", "is.double(c(1.5, 2))"]
C["head"] = ["head(c(\"a\", \"b\", \"c\", \"d\"), 3)", "head(1:10, -3)"]
C["mode"] = ["mode(as.integer(1))", "mode(\"a\")"]
C["typeof"] = ["typeof(as.integer(1))", "typeof(1:2)"]
C["abs"].append("abs(-3, bogus = 2)")
C["grepl"].append('grepl("[0-9]", c("a1", "b"))')
C["grep"].append('grep("^[a-c]", c("apple", "dog", "cat"))')
C["sub"].append('sub("[0-9]+", "#", "a12b")')
C["strsplit"].append('strsplit("a1b2c", "[0-9]")')
C["gsub"].append('gsub("a", "o", "banana")')
C["abs"] = [c for c in C["abs"] if "bogus" not in c]

C["pgamma"].append("pgamma(1.5, 2, scale = 3)")
C["qgamma"].append("qgamma(0.5, 2, scale = 3)")
C["dgamma"].append("dgamma(1.5, 2, scale = 3)")
C["print"].append("for (i in 1:2) print(i)")
