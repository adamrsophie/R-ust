# Supplemental nested and formal-default identity oracle, R 4.6.0.
# Rscript --vanilla scripts/canonical-language-nested-identity-oracle.R
options(keep.source=FALSE, warn=1)
cat("id\tleft_source\tright_source\tidentical\n")
cat("eq_nested_operator\tf(x + y)\tf(`+`(x, y))\t", identical(quote(f(x + y)),quote(f(`+`(x, y)))), "\n", sep="")
cat("eq_default_operator\tfunction(a = x + y) a\tfunction(a = `+`(x, y)) a\t", identical(quote(function(a = x + y) a),quote(function(a = `+`(x, y)) a)), "\n", sep="")
cat("eq_default_nan\tfunction(a = NaN) a\tfunction(a = NaN) a\t", identical(quote(function(a = NaN) a),quote(function(a = NaN) a)), "\n", sep="")
cat("ne_default_nan_na\tfunction(a = NaN) a\tfunction(a = NA_real_) a\t", identical(quote(function(a = NaN) a),quote(function(a = NA_real_) a)), "\n", sep="")
