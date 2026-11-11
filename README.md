# R-ust

R-ust is an implementation of the R language in Rust, targeting compatibility with R 4.6.0.

The project is an early proof of concept. It provides a pure Rust port of R's standalone mathematical library (Rmath), a C ABI for that library, and a parser, evaluator, and command line interface that run a growing subset of the R language.

## Quick start

R-ust requires a stable Rust toolchain (edition 2021).

```bash
cargo build --release
```

Evaluate an expression:

```bash
cargo run -q -p r_ust_cli -- -e 's <- 0; for (i in 1:5) { if (i == 3) next; s <- s + i }; s; pnorm(qnorm(0.7))'
```

```text
[1] 12
[1] 0.7
```

Run a script:

```r
# example.R
scores <- c(72, 88, 95, 61, 79)
grade <- function(x) if (x >= 80) "pass" else "review"
sapply(scores, grade)
mean(scores)
```

```bash
cargo run -q -p r_ust_cli -- example.R
```

```text
[1] "review" "pass" "pass" "review" "review"
[1] 79
```

## Workspace

| Crate | Purpose |
| --- | --- |
| `r_ust_sexp` | Core R object model (SEXP types) |
| `r_ust_parse` | Lexer and parser for R source |
| `r_ust_eval` | Evaluator and built-in functions |
| `r_ust_gc` | Memory management and protection stack |
| `r_ust_nmath` | Pure Rust port of the standalone Rmath library |
| `r_ust_ffi` | C ABI exports of the Rmath surface |
| `r_ust_runtime` | Runtime assembly of the components above |
| `r_ust_cli` | Command line interface |

## Supported language features

**Syntax and evaluation**

- Numeric, logical, character, and `NULL` literals
- Assignment with `<-`, `=`, `->`, `<<-`, and `->>`, including replacement through `[`, `[[`, `$`, and `names<-`
- Control flow: `if`/`else`, `for`, `while`, `repeat`, `break`, `next`, and `return()`
- Closures with lazily evaluated arguments and defaults, `missing()`, `...` forwarding, and exact named and positional argument matching
- Operators: arithmetic, comparison, `:`, `&`, `|`, `&&`, `||`, and `%in%`

**Data structures**

- Atomic vectors, lists, factors (including ordered factors), matrices, and data frames

**Built-in functions**

- Vectors: `c()`, `seq()`, `seq_len()`, `seq_along()`, `rep()`, `append()`, `replace()`, `lengths()`, `ifelse()`, `head()`, `tail()`, `rev()`, `sort()`, `order()`, `rank()`, `diff()`, and cumulative functions
- Matching and sets: `which()`, `match()`, `unique()`, `duplicated()`, `union()`, `intersect()`, `setdiff()`, and `setequal()`
- Iteration and grouping: `lapply()`, `sapply()`, `apply()`, `split()`, `tabulate()`, and `table()`
- Matrices: `matrix()`, `dim()`, `nrow()`, `ncol()`, `t()`, `diag()`, `row()`, `col()`, `lower.tri()`, `upper.tri()`, `rowSums()`, `colMeans()`, `cbind()`, `rbind()`, `crossprod()`, and `tcrossprod()`
- Data frames: `data.frame()`, `with()`, `subset()`, `transform()`, `complete.cases()`, and `na.omit()`
- Strings: `paste()`, `paste0()`, `nchar()`, `toupper()`, `tolower()`, `substr()`, `substring()`, `startsWith()`, `endsWith()`, `grepl()`, `grep()`, `sub()`, `gsub()`, `strsplit()`, `trimws()`, `nzchar()`, `chartr()`, `strtrim()`, and `strrep()`
- Types and coercion: `class()`, `typeof()`, `mode()`, `unlist()`, and the common `as.*()` and `is.*()` functions
- Mathematics and statistics: `sum()`, `prod()`, `mean()`, `median()`, `var()`, `sd()`, `min()`, `max()`, `range()`, `pmin()`, `pmax()`, logarithms and exponentials, trigonometric and hyperbolic functions, rounding, and the gamma, beta, and polygamma families
- Probability distributions: density, distribution, quantile, and random generation functions for the normal, uniform, binomial, Poisson, geometric, negative binomial, hypergeometric, exponential, gamma, beta, log-normal, Weibull, logistic, Cauchy, chi-square, t, F, signed rank, and Wilcoxon rank sum distributions, with noncentral variants of the beta, chi-square, t, and F distributions
- Sampling: `sample()` and `sample.int()`, with and without replacement

## Rmath library

`r_ust_nmath` implements all 154 public functions of R's standalone `Rmath.h` interface in pure Rust. `r_ust_ffi` exports the same functions through a C ABI, built as a static and dynamic library, so existing C code written against Rmath can link against R-ust.

## Known limitations

R-ust does not yet provide an interactive console, reference semantics for environments, general replacement functions, full attribute support (including row names and arrays), formulas, S3 and S4 dispatch, namespaces and package loading, condition handling, connections, graphics, serialization, or the base and recommended packages.

Output formatting differs from R in places: numbers print with up to 12 significant digits instead of R's default of 7, names are not yet shown when printing named vectors, and character vectors are not column aligned.

Several Rmath families use accurate approximations that do not yet reproduce R's results to the last bit:

- Bessel functions use series, reflection, and asymptotic approximations in place of R's Cody and Amos style routines.
- Noncentral beta, F, t, and chi-square distributions use mixture, integration, and bisection methods.
- The Tukey studentized range distribution and quantile use a monotone approximation in place of R's quadrature algorithm.
- Random number generators for the gamma, beta, F, t, binomial, Poisson, hypergeometric, rank, and multinomial families produce valid draws but not R's exact random streams.
- Wilcoxon and signed rank distributions use exact dynamic programming tables; the exported `*_free` functions are no-ops.

## Testing

```bash
cargo test
scripts/check-c-abi.sh
```

`scripts/check-c-abi.sh` builds the static library and links a C program against it to exercise the exported Rmath ABI. It requires a C compiler.

## License

R-ust is free software, distributed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).
