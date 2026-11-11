# R-ust porting status

Reference: R 4.6.0 (2026-04-24). R-ust commit `0c4e40f`. Generated 2026-11-11.

This file records, for every object exported by the R 4.6.0 base distribution and for the core language features, whether R-ust implements it, implements it in part, or lacks it. The R development trunk (revision r90875, R 4.7.0 under development) is checked out in `documentation/R-devel` and has not been compared yet.

## Contents

1. [Summary](#summary)
2. [Method](#method)
3. [Root causes](#root-causes)
4. [Language features](#language-features)
5. [Runtime and infrastructure](#runtime-and-infrastructure)
6. [Rmath C library](#rmath-c-library)
7. [Functions by package](#functions-by-package)
8. [Names in R-ust that R 4.6.0 does not have](#names-in-r-ust-that-r-460-does-not-have)

## Summary

| Package | Objects | Done | Partial | Missing |
| --- | ---: | ---: | ---: | ---: |
| base | 1407 | 155 | 65 | 1187 |
| stats | 465 | 57 | 24 | 384 |
| utils | 237 | 1 | 1 | 235 |
| methods | 365 | 0 | 0 | 365 |
| graphics | 88 | 0 | 0 | 88 |
| grDevices | 137 | 0 | 0 | 137 |
| datasets | 108 | 0 | 0 | 108 |
| tools | 138 | 0 | 0 | 138 |
| parallel | 38 | 0 | 0 | 38 |
| compiler | 9 | 0 | 0 | 9 |
| grid | 249 | 0 | 0 | 249 |
| splines | 13 | 0 | 0 | 13 |
| stats4 | 28 | 0 | 0 | 28 |
| tcltk | 272 | 0 | 0 | 272 |
| **Total** | **3554** | **213** | **90** | **3251** |

Language features: 183 checked, 89 done, 21 partial, 73 missing.

Recommended packages shipped with R (MASS, lattice, Matrix, nlme, survival, boot, cluster, codetools, foreign, KernSmooth, rpart, class, nnet, spatial, mgcv) are not counted above; none of them can run on R-ust yet.

## Method

1. **Inventory.** Every object in the base environment and every export of the other 13 base distribution packages was listed from an installed R 4.6.0, with its formal arguments. For S3 generics the formals of the default method were added, and for `rep` and `c` the documented arguments that live in `...`.
2. **Availability.** Each syntactic name was evaluated in R-ust (`name()` for functions, `name` for data). `object 'name' not found` means Missing. Operators, replacement functions and other non syntactic names were classified by hand from the language feature checks below.
3. **Arguments.** For each available function, the Rust code that implements it (its dispatch arm and the functions it calls, three levels deep) was scanned for string literals naming R's formal arguments. A formal that never appears is listed as *not referenced*; that result is reliable, and because unknown named arguments are silently dropped (RC9), passing such an argument by name has no effect. A formal that does appear is only *referenced*, which does not prove it is honored (for example `rep(each =)` and `sapply(USE.NAMES =)` are referenced or accepted but ignored). The first formal counts as referenced because every function accepts it by position.
4. **Behavior.** One to three representative calls per available function, including mixed width numbers such as `c(10, 9, 100, 2)` for ordering functions, were run in R 4.6.0 with `options(digits = 12)` and in R-ust, and the printed results were compared. Identical output is a match. Output that differs only in layout, or in numbers equal to a relative tolerance of 1e-9, is a formatting difference. Anything else is a value difference.
5. **Language features.** 183 snippets covering syntax, operators, indexing, assignment, control flow, functions, environments, conditions, object systems, types and printing were run the same way.
6. **Operator combinations.** 416 additional expressions combine supported binary operators and unary signs/negation. `check_operators.py` requires matching values under the same printed numeric tolerance, or an error in both runtimes; error message wording is not compared. Inputs and results are recorded in `operators_snips.tsv`, `operators_r_out.txt`, and `operators.tsv`. The reference harness escapes embedded newlines and checks case counts before pairing results.
7. **Missing values.** 592 expressions in `check_missing.py` compare NA and NaN across arithmetic, summaries, unary math, coercion, matching, indexing and matrix summaries. `missing_snips.tsv`, `missing_r_out.txt`, and `missing.tsv` preserve inputs and results. This is a targeted regression corpus, not full-range numerical validation.
8. **Integer types and values.** 707 expressions in `check_integers.py` compare `typeof` and values for mixed arithmetic, boundary literals, sequences, coercion, summaries, simplification and replacement. Inputs and outputs are in `integers_snips.tsv`, `integers_r_out.txt`, and `integers.tsv`. Warning messages and compact storage are not checked.

Status words:

- **Done**: available, no formal argument is unreferenced, and every reference call returned the same values as R. Formatting differences alone do not prevent Done, because printing is tracked as its own feature (RC8). Done rests on a small number of calls, not on exhaustive testing.
- **Partial**: available, but with unreferenced arguments, a value difference, an error, or a known root cause.
- **Missing**: not available in R-ust.
- **Unverified** (Rmath only): implemented in Rust but with no R level reference call in this run.

The scripts and raw results that produced this file are in `documentation/porting-data/`.

## Root causes

Most partial results trace back to a few shared defects. Rows below cite these identifiers.

- **RC1. Numeric comparison fixed; full comparison semantics remain partial.** All six operators compare numeric and logical operands numerically, retain floating point precision, and return `NA` for missing numeric operands. Character operands select string coercion and missing strings propagate `NA`. Remaining gaps include matrix attributes, factor dispatch, locale-sensitive string collation, and number-to-string conversion (RC14). Source: `compare_values` in `crates/r_ust_eval/src/lib.rs`.
- **RC2. Resolved: core double NA representation.** Double vectors use R's NA payload, distinct from ordinary `NaN`. `NA_integer_`, `NA_real_` and `NA_character_` are parsed. Coercion, `is.nan`, printing, indexing, growth, matching, identity, and covered numeric helpers preserve the distinction. Numeric indices retain missing entries and reject mixing negative indices with positive or missing indices. Complex NA remains unsupported with the complex type. See `missing.tsv` for reference comparisons.
- **RC3. Atomic vectors carry no attributes.** Names on atomic vectors are dropped (`c(a = 1)`, `names()`, `unlist()`, `lengths()`, `sapply()`), `table()` returns a named list, row names are renumbered, and `attr()`, `attributes()`, `structure()`, `dimnames` and arrays beyond matrices are unavailable.
- **RC4. Integer values and types implemented; warnings and compact storage remain missing.** Valid decimal `L` literals produce integers. Colon sequences choose integer storage where R does; `seq_len` and `seq_along` return integers for ordinary lengths. Integer `+`, `-`, `*`, cumulative operations, coercion, concatenation, simplification, summaries and replacement preserve or promote types. Arithmetic overflow becomes `NA`, while an overflowing integer `sum` promotes to double. Overflow and invalid `L` qualifier warnings are not emitted. Sequences are materialized, without R's compact ALTREP storage; very large sequences remain unsupported. Other `seq` arguments remain partial. See `integers.tsv` for type and value checks.
- **RC5. Random streams differ from R.** `set.seed()` does not reproduce R's Mersenne Twister seeding, so every random generator and `sample()` produce different draws from R even when the generator algorithm itself matches. Several generator algorithms also differ (see the Rmath section).
- **RC6. Gamma family has no `rate` argument.** R's signature is `dgamma(x, shape, rate = 1, scale = 1/rate)`. R-ust reads the third positional argument as `scale` and silently ignores `rate =` (RC9), so `dgamma(1.5, 2, 3)` and `dgamma(1.5, 2, rate = 3)` both differ from R. Only `scale =` matches R. Affects `dgamma`, `pgamma`, `qgamma`, `rgamma`.
- **RC7. Regular expressions do not support bracket expressions.** Patterns such as `[aeiou]`, `[0-9]+` and `^[a-c]` do not match, so `grepl`, `grep`, `sub`, `gsub` and `strsplit` fail on them. Plain literal patterns work.
- **RC8. Printing differs from R.** Numbers generally print with 12 decimal places (trimmed), and tiny values with 12 significant digits, instead of R's default of 7 significant digits. Vectors are neither column aligned nor formatted to a common number of decimals, long vectors do not wrap with index prefixes, `NULL` prints nothing, empty vectors print as `[1]`, data frame strings print with quotes, and non syntactic list names print without backticks.
- **RC9. Builtins accept unknown named arguments.** `abs(-3, bogus = 2)` returns `3`; R reports an error. Closures do reject unknown arguments.
- **RC10. Builtins are not first class values.** `f <- sum` fails with `object 'sum' not found`; builtins can only be called by name. Backtick quoted names are a parse error.
- **RC11. Resolved: core operator precedence.** `-2^2` evaluates to `-4`, powers associate right, signed exponents work, and `!` binds below comparisons. Special operators bind above multiplication on both sides. `**` aliases `^`. Unparenthesized comparison chains are rejected. Power evaluation uses the existing `R_pow` port for R's infinite and NaN cases.
- **RC12. Visibility is decided only from the top level expression.** Top level assignments are invisible, but `(x <- 5)` prints nothing, and a function whose last expression is an assignment prints its value. R tracks visibility through calls and parentheses.
- **RC13. Polygamma accuracy.** `trigamma()` and `psigamma()` differ from R at about 1e-5 relative error; `digamma()` matches.
- **RC14. Number to string conversion.** `as.character(1e6)` gives `"1000000"`; R gives `"1e+06"` (15 significant digits with R's scientific notation rules).
- **RC15. No output from inside loops or functions.** `print()` returns its argument without printing, and `cat()`, `message()` and `writeLines()` are missing. Only values of top level expressions reach the console, so `for (i in 1:2) print(i)` prints nothing.

## Language features

### Literals

20 checked: 14 done, 1 partial, 5 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Double literal | Done | <code>1.5</code> | <code>[1] 1.5</code> | <code>[1] 1.5</code> |  |
| Integer literal (1L) | Done | <code>x &lt;- 5L; typeof(x)</code> | <code>[1] &quot;integer&quot;</code> | <code>[1] &quot;integer&quot;</code> | Integer storage implemented; invalid L qualifiers do not warn (RC4). |
| Sequence a:b yields integer | Done | <code>typeof(1:3)</code> | <code>[1] &quot;integer&quot;</code> | <code>[1] &quot;integer&quot;</code> |  |
| Hexadecimal literal | Missing | <code>0x1F</code> | <code>[1] 31</code> | <code>ERROR: parse error: expected expression separator</code> | Parse error. |
| Scientific notation | Done | <code>1.5e3</code> | <code>[1] 1500</code> | <code>[1] 1500</code> |  |
| Complex literal | Missing | <code>(2+3i) * 2i</code> | <code>[1] -6+4i</code> | <code>ERROR: parse error: expected RParen, found Ident(&quot; …</code> | Parse error; no complex type. |
| Double-quoted string | Done | <code>&quot;abc&quot;</code> | <code>[1] &quot;abc&quot;</code> | <code>[1] &quot;abc&quot;</code> |  |
| Single-quoted string | Done | <code>&#x27;abc&#x27;</code> | <code>[1] &quot;abc&quot;</code> | <code>[1] &quot;abc&quot;</code> |  |
| String escapes | Done | <code>nchar(&quot;a\tb ⏎ &quot;)</code> | <code>[1] 4</code> | <code>[1] 4</code> |  |
| Unicode escape | Partial | <code>nchar(&quot;\u00e9&quot;)</code> | <code>[1] 1</code> | <code>[1] 5</code> | `\u` escapes are not decoded. |
| Raw string literal | Missing | <code>r&quot;(C:\path)&quot;</code> | <code>[1] &quot;C:\\path&quot;</code> | <code>ERROR: parse error: expected expression separator</code> | Parse error. |
| TRUE / FALSE / T / F | Done | <code>c(TRUE, FALSE, T, F)</code> | <code>[1]  TRUE FALSE  TRUE FALSE</code> | <code>[1] TRUE FALSE TRUE FALSE</code> | Values match; printing differs (RC8). |
| NULL | Done | <code>is.null(NULL)</code> | <code>[1] TRUE</code> | <code>[1] TRUE</code> |  |
| NA | Done | <code>NA</code> | <code>[1] NA</code> | <code>[1] NA</code> |  |
| NA distinct from NaN | Done | <code>is.nan(c(NA, NaN))</code> | <code>[1] FALSE  TRUE</code> | <code>[1] FALSE TRUE</code> | Values match; printing differs (RC8). |
| Typed NA constants | Done | <code>c(typeof(NA_integer_), typeof(NA_real_), typeof(NA_character_))</code> | <code>[1] &quot;integer&quot;   &quot;double&quot;    &quot;character&quot;</code> | <code>[1] &quot;integer&quot; &quot;double&quot; &quot;character&quot;</code> | Atomic integer, double and character NA constants are implemented. Complex NA remains missing with the complex type. |
| Inf and NaN | Done | <code>c(Inf, -Inf, NaN)</code> | <code>[1]  Inf -Inf  NaN</code> | <code>[1] Inf -Inf NaN</code> | Values match; printing differs (RC8). |
| Backtick-quoted names | Missing | <code>`my var` &lt;- 3; `my var` + 1</code> | <code>[1] 4</code> | <code>ERROR: parse error: unexpected character &#x27;`&#x27; at by …</code> | Parse error (RC10). |
| Comments | Done | <code>1 + 1 # comment</code> | <code>[1] 2</code> | <code>[1] 2</code> |  |
| Built-in constants (pi, letters, LETTERS, month.name) | Missing | <code>c(pi, length(letters), length(LETTERS), length(month.name))</code> | <code>[1]  3.14159265359 26.00000000000 26.00000000000 1 …</code> | <code>ERROR: object &#x27;pi&#x27; not found</code> | None of these constants are defined. |

### Operators

38 checked: 28 done, 1 partial, 9 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Arithmetic + - * / | Done | <code>c(7 + 2, 7 - 2, 7 * 2, 7 / 2)</code> | <code>[1]  9.0  5.0 14.0  3.5</code> | <code>[1] 9 5 14 3.5</code> | Values match; printing differs (RC8). |
| Power ^ | Done | <code>2^10</code> | <code>[1] 1024</code> | <code>[1] 1024</code> |  |
| Power alias ** | Done | <code>2 ** 3</code> | <code>[1] 8</code> | <code>[1] 8</code> |  |
| Integer division %/% and modulo %% | Missing | <code>c(17 %/% 5, 17 %% 5, -7 %% 3)</code> | <code>[1] 3 2 2</code> | <code>ERROR: parse error: special operator %/% is not im …</code> |  |
| Unary minus and plus | Done | <code>-(3) + +2</code> | <code>[1] -1</code> | <code>[1] -1</code> |  |
| Sequence operator : | Done | <code>3:-1</code> | <code>[1]  3  2  1  0 -1</code> | <code>[1] 3 2 1 0 -1</code> | Values match; printing differs (RC8). |
| Comparison operators | Done | <code>c(1 &lt; 2, 2 &lt;= 2, 3 &gt; 4, 4 &gt;= 5, 1 == 1, 1 != 1)</code> | <code>[1]  TRUE  TRUE FALSE FALSE  TRUE FALSE</code> | <code>[1] TRUE TRUE FALSE FALSE TRUE FALSE</code> | Values match; printing differs (RC8). |
| Numeric comparison of multi-digit numbers | Done | <code>c(10 &lt; 2, 9 &lt; 10, 100 &gt; 20, 2.5 &lt; 10)</code> | <code>[1] FALSE  TRUE  TRUE  TRUE</code> | <code>[1] FALSE TRUE TRUE TRUE</code> | Values match; printing differs (RC8). |
| Equality of logical and numeric | Done | <code>TRUE == 1</code> | <code>[1] TRUE</code> | <code>[1] TRUE</code> |  |
| Floating point equality | Done | <code>0.1 + 0.2 == 0.3</code> | <code>[1] FALSE</code> | <code>[1] FALSE</code> |  |
| Comparison with NA yields NA | Done | <code>c(NA == 1, NaN == NaN)</code> | <code>[1] NA NA</code> | <code>[1] NA NA</code> |  |
| String comparison | Done | <code>c(&quot;apple&quot; &lt; &quot;banana&quot;, &quot;b&quot; &gt; &quot;a&quot;)</code> | <code>[1] TRUE TRUE</code> | <code>[1] TRUE TRUE</code> |  |
| Vectorized logical ! &amp; &#124; | Done | <code>!c(TRUE, FALSE) &amp; c(TRUE, TRUE) &#124; c(FALSE, FALSE)</code> | <code>[1] FALSE  TRUE</code> | <code>[1] FALSE TRUE</code> | Values match; printing differs (RC8). |
| Short-circuit &amp;&amp; &#124;&#124; | Done | <code>FALSE &amp;&amp; stop(&#x27;no&#x27;); TRUE &#124;&#124; stop(&#x27;no&#x27;)</code> | <code>[1] FALSE ⏎ [1] TRUE</code> | <code>[1] FALSE ⏎ [1] TRUE</code> |  |
| %in% | Done | <code>c(1, 5) %in% 1:3</code> | <code>[1]  TRUE FALSE</code> | <code>[1] TRUE FALSE</code> | Values match; printing differs (RC8). |
| Matrix product %*% | Missing | <code>matrix(1:4, 2) %*% matrix(1:4, 2)</code> | <code>[,1] [,2] ⏎ [1,]    7   15 ⏎ [2,]   10   22</code> | <code>ERROR: parse error: special operator %*% is not im …</code> |  |
| Outer product %o% | Missing | <code>1:2 %o% 1:3</code> | <code>[,1] [,2] [,3] ⏎ [1,]    1    2    3 ⏎ [2,]    2 …</code> | <code>ERROR: parse error: special operator %o% is not im …</code> |  |
| User-defined infix %op% | Missing | <code>&quot;%+%&quot; &lt;- function(a, b) paste(a, b); &quot;x&quot; %+% &quot;y&quot;</code> | <code>[1] &quot;x y&quot;</code> | <code>ERROR: parse error: left side of assignment must b …</code> | Defining `%op%` needs a quoted or backtick name, which does not parse; `%op%` calls are not implemented. |
| Native pipe &#124;&gt; | Missing | <code>c(1, 4, 9) &#124;&gt; sqrt() &#124;&gt; sum()</code> | <code>[1] 6</code> | <code>ERROR: parse error: unexpected token Greater</code> |  |
| Lambda shorthand \(x) | Missing | <code>sapply(1:3, \(x) x * 2)</code> | <code>[1] 2 4 6</code> | <code>ERROR: parse error: unexpected character &#x27;\\&#x27; at b …</code> |  |
| Formula ~ | Missing | <code>f &lt;- y ~ x + z; class(f)</code> | <code>[1] &quot;formula&quot;</code> | <code>ERROR: parse error: unexpected character &#x27;~&#x27; at by …</code> |  |
| Namespace access :: | Missing | <code>base::sum(1:3)</code> | <code>[1] 6</code> | <code>ERROR: parse error: unexpected token Colon</code> |  |
| Internal access ::: | Missing | <code>stats:::Pillai</code> | <code>function (eig, q, df.res)  ⏎ { ⏎     test &lt;- sum(e …</code> | <code>ERROR: parse error: unexpected token Colon</code> |  |
| Operator precedence | Done | <code>-2^2 + 3 * 2 - 1 / 2</code> | <code>[1] 1.5</code> | <code>[1] 1.5</code> |  |
| Power associativity and signed exponents | Done | <code>c(2^3^2, 2^-2, -2^-2, (-2)^2)</code> | <code>[1] 512.00   0.25  -0.25   4.00</code> | <code>[1] 512 0.25 -0.25 4</code> | Values match; printing differs (RC8). |
| Power special values | Done | <code>c(1^NaN, NaN^0, is.nan((-1)^Inf), is.nan((-Inf)^0.5))</code> | <code>[1] 1 1 1 1</code> | <code>[1] 1 1 1 1</code> |  |
| Negation precedence | Done | <code>!1:3 %in% 2:3</code> | <code>[1]  TRUE FALSE FALSE</code> | <code>[1] TRUE FALSE FALSE</code> | Values match; printing differs (RC8). |
| Special operator right of multiplication | Done | <code>2 * 3 %in% 6</code> | <code>[1] 0</code> | <code>[1] 0</code> |  |
| Logical operator precedence | Done | <code>TRUE &#124;&#124; FALSE &#124; c(FALSE, FALSE); FALSE &amp;&amp; TRUE &amp; c(TRUE, TRUE)</code> | <code>[1] TRUE TRUE ⏎ [1] FALSE FALSE</code> | <code>[1] TRUE TRUE ⏎ [1] FALSE FALSE</code> |  |
| All numeric comparison operators | Done | <code>c(10 == 2, 10 != 2, 10 &lt; 2, 10 &lt;= 2, 10 &gt; 2, 10 &gt;= 2)</code> | <code>[1] FALSE  TRUE FALSE FALSE  TRUE  TRUE</code> | <code>[1] FALSE TRUE FALSE FALSE TRUE TRUE</code> | Values match; printing differs (RC8). |
| Numeric comparison preserves precision | Done | <code>c(1e-13 == 0, 0 == ceiling(-0.25), Inf == Inf)</code> | <code>[1] FALSE  TRUE  TRUE</code> | <code>[1] FALSE TRUE TRUE</code> | Values match; printing differs (RC8). |
| Character comparison propagates NA | Done | <code>c(NA, &quot;x&quot;) == &quot;x&quot;</code> | <code>[1]   NA TRUE</code> | <code>[1] NA TRUE</code> | Values match; printing differs (RC8). |
| Unary signs preserve integer storage | Done | <code>c(typeof(+TRUE), typeof(-as.integer(3))); 2 / -!3</code> | <code>[1] &quot;integer&quot; &quot;integer&quot; ⏎ [1] Inf</code> | <code>[1] &quot;integer&quot; &quot;integer&quot; ⏎ [1] Inf</code> |  |
| Missing double arithmetic | Done | <code>c(NA_real_ + NaN, 1^NA_real_, NA_real_^0, NA_real_^2)</code> | <code>[1] NA  1  1 NA</code> | <code>[1] NA 1 1 NA</code> | Values match; printing differs (RC8). |
| NA and NaN matching | Done | <code>match(c(NA_real_, NaN), c(NaN, NA_real_))</code> | <code>[1] 2 1</code> | <code>[1] 2 1</code> |  |
| Vector recycling | Done | <code>1:6 + 1:2</code> | <code>[1] 2 4 4 6 6 8</code> | <code>[1] 2 4 4 6 6 8</code> |  |
| Integer overflow to NA | Partial | <code>as.integer(2147483647) + as.integer(1)</code> | <code>[1] NA</code> | <code>[1] NA</code> | Value matches; overflow warnings are missing (RC4). |
| Integer sum promotes on overflow | Done | <code>c(typeof(sum(1:3)), typeof(sum(c(2147483647L, 1L))))</code> | <code>[1] &quot;integer&quot; &quot;double&quot;</code> | <code>[1] &quot;integer&quot; &quot;double&quot;</code> |  |

### Indexing

13 checked: 9 done, 2 partial, 2 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Positive index [ | Done | <code>x &lt;- c(10, 20, 30); x[c(1, 3)]</code> | <code>[1] 10 30</code> | <code>[1] 10 30</code> |  |
| Negative index | Done | <code>x &lt;- c(10, 20, 30); x[-1]</code> | <code>[1] 20 30</code> | <code>[1] 20 30</code> |  |
| Logical index | Done | <code>x &lt;- c(10, 20, 30); x[x &gt; 15]</code> | <code>[1] 20 30</code> | <code>[1] 20 30</code> |  |
| Character (name) index | Missing | <code>x &lt;- c(a = 1, b = 2); x[&quot;b&quot;]</code> | <code>b  ⏎ 2</code> | <code>ERROR: type error: character subsetting is only im …</code> | Works for lists, not atomic vectors (RC3). |
| Out of bounds index | Done | <code>x &lt;- 1:3; x[5]</code> | <code>[1] NA</code> | <code>[1] NA</code> |  |
| Numeric missing indices retain positions | Done | <code>x &lt;- c(1, NaN); x[c(2, 3, NA_real_)]</code> | <code>[1] NaN  NA  NA</code> | <code>[1] NaN NA NA</code> | Values match; printing differs (RC8). |
| Matrix [i, j] | Done | <code>m &lt;- matrix(1:6, 2); m[2, 3]</code> | <code>[1] 6</code> | <code>[1] 6</code> |  |
| Matrix row with drop | Done | <code>m &lt;- matrix(1:6, 2); m[1, ]</code> | <code>[1] 1 3 5</code> | <code>[1] 1 3 5</code> |  |
| drop = FALSE | Missing | <code>m &lt;- matrix(1:6, 2); dim(m[1, , drop = FALSE])</code> | <code>[1] 1 3</code> | <code>ERROR: parse error: unexpected token Comma</code> | Empty index followed by a named argument is a parse error. |
| List [[ and $ | Done | <code>l &lt;- list(a = 1, b = &quot;x&quot;); c(l[[1]], l$b)</code> | <code>[1] &quot;1&quot; &quot;x&quot;</code> | <code>[1] &quot;1&quot; &quot;x&quot;</code> |  |
| Partial matching with $ | Partial | <code>l &lt;- list(alpha = 1); l$al</code> | <code>[1] 1</code> | <code></code> | Returns `NULL`. |
| Recursive [[ ]] indexing | Partial | <code>l &lt;- list(a = list(b = 42)); l[[c(1, 1)]]</code> | <code>[1] 42</code> | <code>$b ⏎ [1] 42</code> | Returns the first level only. |
| Data frame [i, j] | Done | <code>df &lt;- data.frame(x = 1:3, y = 4:6); df[2, &quot;y&quot;]</code> | <code>[1] 5</code> | <code>[1] 5</code> |  |

### Assignment

17 checked: 9 done, 2 partial, 6 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Left assignment &lt;- | Done | <code>x &lt;- 5; x</code> | <code>[1] 5</code> | <code>[1] 5</code> |  |
| Equals assignment = | Done | <code>x = 5; x</code> | <code>[1] 5</code> | <code>[1] 5</code> |  |
| Right assignment -&gt; | Done | <code>5 -&gt; x; x</code> | <code>[1] 5</code> | <code>[1] 5</code> |  |
| Superassignment &lt;&lt;- | Partial | <code>f &lt;- function() x &lt;&lt;- 9; f(); x</code> | <code>[1] 9</code> | <code>[1] 9 ⏎ [1] 9</code> | Assignment works; the function call result also prints (RC12). |
| Assignment returns value invisibly | Done | <code>x &lt;- 5</code> | <code></code> | <code></code> |  |
| Printing via parentheses | Partial | <code>(x &lt;- 5)</code> | <code>[1] 5</code> | <code></code> | RC12. |
| Subset replacement x[i] &lt;- | Done | <code>x &lt;- c(1, 2, 3, 4, 5); x[2] &lt;- 0; x</code> | <code>[1] 1 0 3 4 5</code> | <code>[1] 1 0 3 4 5</code> |  |
| Growing a vector by index | Done | <code>x &lt;- c(1, 2); x[5] &lt;- 9; x</code> | <code>[1]  1  2 NA NA  9</code> | <code>[1] 1 2 NA NA 9</code> | Values match; printing differs (RC8). |
| Double replacement promotes integer vector | Done | <code>x &lt;- 1:3; x[1] &lt;- 99; typeof(x); x</code> | <code>[1] &quot;double&quot; ⏎ [1] 99  2  3</code> | <code>[1] &quot;double&quot; ⏎ [1] 99 2 3</code> | Values match; printing differs (RC8). |
| List element replacement $&lt;- | Done | <code>l &lt;- list(a = 1); l$b &lt;- 2; names(l)</code> | <code>[1] &quot;a&quot; &quot;b&quot;</code> | <code>[1] &quot;a&quot; &quot;b&quot;</code> |  |
| names&lt;- replacement | Missing | <code>x &lt;- 1:2; names(x) &lt;- c(&quot;a&quot;, &quot;b&quot;); x</code> | <code>a b  ⏎ 1 2</code> | <code>ERROR: type error: cannot set names on value of ty …</code> | Atomic vectors cannot hold names (RC3). |
| dim&lt;- replacement | Missing | <code>x &lt;- 1:6; dim(x) &lt;- c(2, 3); x</code> | <code>[,1] [,2] [,3] ⏎ [1,]    1    3    5 ⏎ [2,]    2 …</code> | <code>ERROR: parse error: left side of assignment must b …</code> |  |
| levels&lt;- replacement | Missing | <code>f &lt;- factor(c(&quot;a&quot;, &quot;b&quot;)); levels(f) &lt;- c(&quot;x&quot;, &quot;y&quot;); f</code> | <code>[1] x y ⏎ Levels: x y</code> | <code>ERROR: parse error: left side of assignment must b …</code> |  |
| attr&lt;- replacement | Missing | <code>x &lt;- 1; attr(x, &quot;u&quot;) &lt;- &quot;m&quot;; attributes(x)</code> | <code>$u ⏎ [1] &quot;m&quot; ⏎ </code> | <code>ERROR: parse error: left side of assignment must b …</code> |  |
| User-defined replacement function | Missing | <code>&quot;second&lt;-&quot; &lt;- function(x, value) { x[2] &lt;- value; x }; x &lt;- c(1, 2, 3) …</code> | <code>[1] 1 0 3</code> | <code>ERROR: parse error: left side of assignment must b …</code> | Defining `name<-` needs a quoted or backtick name, which does not parse (RC10). |
| Multiple assignment chain | Done | <code>a &lt;- b &lt;- 3; a + b</code> | <code>[1] 6</code> | <code>[1] 6</code> |  |
| assign() and get() | Missing | <code>assign(&quot;v&quot;, 4); get(&quot;v&quot;)</code> | <code>[1] 4</code> | <code>ERROR: object &#x27;assign&#x27; not found</code> |  |

### Control flow

12 checked: 9 done, 0 partial, 3 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| if / else | Done | <code>x &lt;- 3; if (x &gt; 2) &quot;big&quot; else &quot;small&quot;</code> | <code>[1] &quot;big&quot;</code> | <code>[1] &quot;big&quot;</code> |  |
| else if chain | Done | <code>x &lt;- 0; if (x &gt; 0) 1 else if (x &lt; 0) -1 else 0</code> | <code>[1] 0</code> | <code>[1] 0</code> |  |
| if with NA condition errors | Done | <code>if (NA) 1</code> | <code>ERROR: missing value where TRUE/FALSE needed</code> | <code>ERROR: domain error: missing value where TRUE/FALS …</code> |  |
| for over vector | Done | <code>s &lt;- 0; for (i in 1:4) s &lt;- s + i; s</code> | <code>[1] 10</code> | <code>[1] 10</code> |  |
| for over list | Done | <code>out &lt;- c(); for (e in list(1, 2)) out &lt;- c(out, e * 10); out</code> | <code>[1] 10 20</code> | <code>[1] 10 20</code> |  |
| while | Done | <code>i &lt;- 0; while (i &lt; 3) i &lt;- i + 1; i</code> | <code>[1] 3</code> | <code>[1] 3</code> |  |
| repeat / break | Done | <code>i &lt;- 0; repeat { i &lt;- i + 1; if (i &gt;= 4) break }; i</code> | <code>[1] 4</code> | <code>[1] 4</code> |  |
| next | Done | <code>s &lt;- 0; for (i in 1:5) { if (i == 2 &#124;&#124; i == 4) next; s &lt;- s + i }; s</code> | <code>[1] 9</code> | <code>[1] 9</code> |  |
| switch on character | Missing | <code>switch(&quot;b&quot;, a = &quot;A&quot;, b = &quot;B&quot;, &quot;other&quot;)</code> | <code>[1] &quot;B&quot;</code> | <code>ERROR: object &#x27;switch&#x27; not found</code> |  |
| switch fall-through | Missing | <code>switch(&quot;a&quot;, a = , b = &quot;AB&quot;)</code> | <code>[1] &quot;AB&quot;</code> | <code>ERROR: parse error: unexpected token Comma</code> |  |
| switch on number | Missing | <code>switch(2, &quot;one&quot;, &quot;two&quot;, &quot;three&quot;)</code> | <code>[1] &quot;two&quot;</code> | <code>ERROR: object &#x27;switch&#x27; not found</code> |  |
| for loop value is invisible NULL | Done | <code>x &lt;- for (i in 1:2) i; is.null(x)</code> | <code>[1] TRUE</code> | <code>[1] TRUE</code> |  |

### Functions

27 checked: 13 done, 3 partial, 11 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Closure definition and call | Done | <code>f &lt;- function(x, y) x + y; f(1, 2)</code> | <code>[1] 3</code> | <code>[1] 3</code> |  |
| Unknown named argument is an error | Done | <code>f &lt;- function(x) x; f(1, bogus = 2)</code> | <code>ERROR: unused argument (bogus = 2)</code> | <code>ERROR: argument error: unused argument bogus</code> |  |
| Unknown named argument to a builtin is an error | Partial | <code>abs(-3, bogus = 2)</code> | <code>ERROR: 2 arguments passed to &#x27;abs&#x27; which requires …</code> | <code>[1] 3</code> | RC9. |
| Default arguments | Done | <code>f &lt;- function(x, y = 10) x + y; f(1)</code> | <code>[1] 11</code> | <code>[1] 11</code> |  |
| Default referring to other argument | Done | <code>f &lt;- function(x, y = x * 2) y; f(4)</code> | <code>[1] 8</code> | <code>[1] 8</code> |  |
| Named argument matching | Done | <code>f &lt;- function(a, b) a - b; f(b = 1, a = 5)</code> | <code>[1] 4</code> | <code>[1] 4</code> |  |
| Partial argument matching | Missing | <code>f &lt;- function(value, other) value; f(val = 3, other = 1)</code> | <code>[1] 3</code> | <code>ERROR: argument error: unused argument val</code> | Only exact names and positions match. |
| Lazy evaluation | Done | <code>f &lt;- function(x) 1; f(stop(&quot;not evaluated&quot;))</code> | <code>[1] 1</code> | <code>[1] 1</code> |  |
| missing() | Done | <code>f &lt;- function(x) missing(x); f()</code> | <code>[1] TRUE</code> | <code>[1] TRUE</code> |  |
| Variadic ... forwarding | Done | <code>f &lt;- function(...) sum(...); f(1, 2, 3)</code> | <code>[1] 6</code> | <code>[1] 6</code> |  |
| ..1 and ...length() | Missing | <code>f &lt;- function(...) c(..1, ...length()); f(7, 8, 9)</code> | <code>[1] 7 3</code> | <code>ERROR: object &#x27;..1&#x27; not found</code> |  |
| list(...) with names | Done | <code>f &lt;- function(...) names(list(...)); f(a = 1, b = 2)</code> | <code>[1] &quot;a&quot; &quot;b&quot;</code> | <code>[1] &quot;a&quot; &quot;b&quot;</code> |  |
| Explicit return() | Done | <code>f &lt;- function() { return(1); 2 }; f()</code> | <code>[1] 1</code> | <code>[1] 1</code> |  |
| Implicit return of last value | Done | <code>f &lt;- function() { 1; 2 }; f()</code> | <code>[1] 2</code> | <code>[1] 2</code> |  |
| invisible() | Missing | <code>f &lt;- function() invisible(7); f()</code> | <code></code> | <code>ERROR: object &#x27;invisible&#x27; not found</code> |  |
| Recursion | Done | <code>fib &lt;- function(n) if (n &lt; 2) n else fib(n - 1) + fib(n - 2); fib(9)</code> | <code>[1] 34</code> | <code>[1] 34</code> |  |
| Closures capture environment | Partial | <code>make &lt;- function() { n &lt;- 0; function() { n &lt;&lt;- n + 1; n } }; counter …</code> | <code>[1] 1 ⏎ [1] 2</code> | <code>[1] 1 ⏎ [1] 1</code> | `<<-` inside a closure does not update the enclosing function environment. |
| Functions as values | Missing | <code>f &lt;- sum; f(1:4)</code> | <code>[1] 10</code> | <code>ERROR: object &#x27;sum&#x27; not found</code> | RC10. |
| Anonymous function call | Done | <code>(function(x) x^2)(4)</code> | <code>[1] 16</code> | <code>[1] 16</code> |  |
| Function printing | Partial | <code>f &lt;- function(x) x + 1; f</code> | <code>function (x)  ⏎ x + 1 ⏎ &lt;environment: 0x7cd3b43118 …</code> | <code>function(x)</code> | Prints only the header line. |
| do.call | Missing | <code>do.call(paste, list(&quot;a&quot;, &quot;b&quot;, sep = &quot;-&quot;))</code> | <code>[1] &quot;a-b&quot;</code> | <code>ERROR: object &#x27;do.call&#x27; not found</code> |  |
| match.arg | Missing | <code>f &lt;- function(type = c(&quot;linear&quot;, &quot;quad&quot;)) match.arg(type); f()</code> | <code>[1] &quot;linear&quot;</code> | <code>ERROR: object &#x27;match.arg&#x27; not found</code> |  |
| on.exit | Missing | <code>f &lt;- function() { on.exit(cat(&quot;bye ⏎ &quot;)); 1 }; f()</code> | <code>bye ⏎ [1] 1</code> | <code>ERROR: object &#x27;on.exit&#x27; not found</code> |  |
| sys.call / match.call | Missing | <code>f &lt;- function(x, y) match.call(); f(1, y = 2)</code> | <code>f(x = 1, y = 2)</code> | <code>ERROR: object &#x27;match.call&#x27; not found</code> |  |
| Recall | Missing | <code>fact &lt;- function(n) if (n &lt;= 1) 1 else n * Recall(n - 1); fact(5)</code> | <code>[1] 120</code> | <code>ERROR: object &#x27;Recall&#x27; not found</code> |  |
| Vectorize / Map / Reduce / Filter | Missing | <code>Reduce(function(a, b) a + b, 1:4, accumulate = TRUE)</code> | <code>[1]  1  3  6 10</code> | <code>ERROR: object &#x27;Reduce&#x27; not found</code> |  |
| vapply / mapply | Missing | <code>vapply(1:3, function(i) i * 2, numeric(1))</code> | <code>[1] 2 4 6</code> | <code>ERROR: object &#x27;vapply&#x27; not found</code> |  |

### Environments

6 checked: 1 done, 0 partial, 5 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| environment() and new.env() | Missing | <code>e &lt;- new.env(); assign(&quot;z&quot;, 1, envir = e); exists(&quot;z&quot;, envir = e)</code> | <code>[1] TRUE</code> | <code>ERROR: object &#x27;new.env&#x27; not found</code> |  |
| local() | Missing | <code>local({ a &lt;- 2; a * 3 })</code> | <code>[1] 6</code> | <code>ERROR: object &#x27;local&#x27; not found</code> |  |
| Lexical scoping | Done | <code>x &lt;- 1; f &lt;- function() x; g &lt;- function() { x &lt;- 2; f() }; g()</code> | <code>[1] 1</code> | <code>[1] 1</code> |  |
| Dynamic get via parent.frame | Missing | <code>f &lt;- function() parent.frame(); identical(f(), globalenv())</code> | <code>[1] FALSE</code> | <code>ERROR: object &#x27;parent.frame&#x27; not found</code> |  |
| Environment as mutable object | Missing | <code>e &lt;- new.env(); f &lt;- function(env) env$v &lt;- 5; f(e); e$v</code> | <code>[1] 5</code> | <code>ERROR: object &#x27;new.env&#x27; not found</code> |  |
| sys.function / environment(fn) | Missing | <code>f &lt;- function() 1; environmentName(environment(f))</code> | <code>[1] &quot;&quot;</code> | <code>ERROR: object &#x27;environmentName&#x27; not found</code> |  |

### Conditions

9 checked: 0 done, 0 partial, 9 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| stop() error | Missing | <code>stop(&quot;boom&quot;)</code> | <code>ERROR: boom</code> | <code>ERROR: object &#x27;stop&#x27; not found</code> |  |
| warning() | Missing | <code>f &lt;- function() { warning(&quot;careful&quot;); 1 }; f()</code> | <code>[1] 1</code> | <code>ERROR: object &#x27;warning&#x27; not found</code> |  |
| message() | Missing | <code>message(&quot;note&quot;)</code> | <code></code> | <code>ERROR: object &#x27;message&#x27; not found</code> |  |
| tryCatch error handler | Missing | <code>tryCatch(stop(&quot;boom&quot;), error = function(e) conditionMessage(e))</code> | <code>[1] &quot;boom&quot;</code> | <code>ERROR: object &#x27;tryCatch&#x27; not found</code> |  |
| tryCatch finally | Missing | <code>tryCatch(1, finally = cat(&quot;done ⏎ &quot;))</code> | <code>done ⏎ [1] 1</code> | <code>ERROR: object &#x27;tryCatch&#x27; not found</code> |  |
| try() | Missing | <code>r &lt;- try(log(-1), silent = TRUE); r</code> | <code>[1] NaN</code> | <code>ERROR: object &#x27;try&#x27; not found</code> |  |
| withCallingHandlers | Missing | <code>withCallingHandlers({ warning(&quot;w&quot;); &quot;after&quot; }, warning = function(w) i …</code> | <code>[1] &quot;after&quot;</code> | <code>ERROR: object &#x27;withCallingHandlers&#x27; not found</code> |  |
| Custom condition classes | Missing | <code>tryCatch(stop(structure(class = c(&quot;myError&quot;, &quot;error&quot;, &quot;condition&quot;), li …</code> | <code>[1] &quot;caught&quot;</code> | <code>ERROR: object &#x27;tryCatch&#x27; not found</code> |  |
| stopifnot | Missing | <code>stopifnot(TRUE, 1 == 1); &quot;ok&quot;</code> | <code>[1] &quot;ok&quot;</code> | <code>ERROR: object &#x27;stopifnot&#x27; not found</code> |  |

### OOP

9 checked: 0 done, 0 partial, 9 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| S3 class attribute | Missing | <code>x &lt;- structure(1, class = &quot;thing&quot;); class(x)</code> | <code>[1] &quot;thing&quot;</code> | <code>ERROR: object &#x27;structure&#x27; not found</code> |  |
| S3 UseMethod dispatch | Missing | <code>area &lt;- function(s) UseMethod(&quot;area&quot;); area.square &lt;- function(s) s$si …</code> | <code>[1] 9</code> | <code>ERROR: object &#x27;UseMethod&#x27; not found</code> |  |
| S3 print method | Missing | <code>print.thing &lt;- function(x, ...) cat(&quot;&lt;thing&gt; ⏎ &quot;); structure(1, class …</code> | <code>[1] 1 ⏎ attr(,&quot;class&quot;) ⏎ [1] &quot;thing&quot;</code> | <code>ERROR: object &#x27;structure&#x27; not found</code> |  |
| NextMethod | Missing | <code>f &lt;- function(x) UseMethod(&quot;f&quot;); f.a &lt;- function(x) c(&quot;a&quot;, NextMethod( …</code> | <code>[1] &quot;a&quot;       &quot;default&quot;</code> | <code>ERROR: object &#x27;UseMethod&#x27; not found</code> |  |
| inherits() | Missing | <code>inherits(data.frame(), &quot;data.frame&quot;)</code> | <code>[1] TRUE</code> | <code>ERROR: object &#x27;inherits&#x27; not found</code> |  |
| unclass() | Missing | <code>unclass(factor(c(&quot;a&quot;, &quot;b&quot;)))</code> | <code>[1] 1 2 ⏎ attr(,&quot;levels&quot;) ⏎ [1] &quot;a&quot; &quot;b&quot;</code> | <code>ERROR: object &#x27;unclass&#x27; not found</code> |  |
| S4 setClass / new / slots | Missing | <code>setClass(&quot;P&quot;, representation(x = &quot;numeric&quot;)); p &lt;- new(&quot;P&quot;, x = 3); p@ …</code> | <code>[1] 3</code> | <code>ERROR: parse error: unexpected character &#x27;@&#x27; at by …</code> |  |
| S4 generics and methods | Missing | <code>setGeneric(&quot;area2&quot;, function(s) standardGeneric(&quot;area2&quot;)); setClass(&quot;S …</code> | <code>[1] &quot;area2&quot; ⏎ [1] 9</code> | <code>ERROR: parse error: unexpected character &#x27;@&#x27; at by …</code> |  |
| Reference classes (setRefClass) | Missing | <code>Acc &lt;- setRefClass(&quot;Acc&quot;, fields = list(total = &quot;numeric&quot;)); a &lt;- Acc$ …</code> | <code>[1] 1</code> | <code>ERROR: object &#x27;setRefClass&#x27; not found</code> |  |

### Types

16 checked: 3 done, 1 partial, 12 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| typeof for basic types | Done | <code>c(typeof(1), typeof(as.integer(1)), typeof(&quot;a&quot;), typeof(TRUE), typeof( …</code> | <code>[1] &quot;double&quot;    &quot;integer&quot;   &quot;character&quot; &quot;logical&quot; …</code> | <code>[1] &quot;double&quot; &quot;integer&quot; &quot;character&quot; &quot;logical&quot; &quot;NULL …</code> | Values match; printing differs (RC8). |
| Complex vectors | Missing | <code>Mod(3+4i)</code> | <code>[1] 5</code> | <code>ERROR: parse error: expected RParen, found Ident(&quot; …</code> |  |
| Raw vectors | Missing | <code>as.raw(c(1, 255))</code> | <code>[1] 01 ff</code> | <code>ERROR: object &#x27;as.raw&#x27; not found</code> |  |
| Language objects quote() | Missing | <code>quote(x + y)</code> | <code>x + y</code> | <code>ERROR: object &#x27;quote&#x27; not found</code> |  |
| eval(parse(text = )) | Missing | <code>eval(parse(text = &quot;1 + 2&quot;))</code> | <code>[1] 3</code> | <code>ERROR: object &#x27;eval&#x27; not found</code> |  |
| substitute / deparse | Missing | <code>f &lt;- function(x) deparse(substitute(x)); f(a + b)</code> | <code>[1] &quot;a + b&quot;</code> | <code>ERROR: object &#x27;deparse&#x27; not found</code> |  |
| bquote | Missing | <code>bquote(x + .(1 + 2))</code> | <code>x + 3</code> | <code>ERROR: object &#x27;bquote&#x27; not found</code> |  |
| expression() | Missing | <code>length(expression(1, 2 + 3))</code> | <code>[1] 2</code> | <code>ERROR: object &#x27;expression&#x27; not found</code> |  |
| Symbols as.name | Missing | <code>class(as.name(&quot;x&quot;))</code> | <code>[1] &quot;name&quot;</code> | <code>ERROR: object &#x27;as.name&#x27; not found</code> |  |
| Type coercion hierarchy in c() | Done | <code>c(TRUE, 2.5, &quot;a&quot;)</code> | <code>[1] &quot;TRUE&quot; &quot;2.5&quot;  &quot;a&quot;</code> | <code>[1] &quot;TRUE&quot; &quot;2.5&quot; &quot;a&quot;</code> | Values match; printing differs (RC8). |
| Implicit logical to numeric | Done | <code>sum(c(TRUE, TRUE, FALSE))</code> | <code>[1] 2</code> | <code>[1] 2</code> |  |
| Attributes and structure() | Missing | <code>attributes(structure(1:4, dim = c(2, 2), note = &quot;n&quot;))</code> | <code>$dim ⏎ [1] 2 2 ⏎  ⏎ $note ⏎ [1] &quot;n&quot; ⏎ </code> | <code>ERROR: object &#x27;attributes&#x27; not found</code> |  |
| Arrays with more than two dimensions | Missing | <code>dim(array(1:24, dim = c(2, 3, 4)))</code> | <code>[1] 2 3 4</code> | <code>ERROR: object &#x27;array&#x27; not found</code> |  |
| Matrix with dimnames | Partial | <code>matrix(1:4, 2, dimnames = list(c(&quot;r1&quot;, &quot;r2&quot;), c(&quot;c1&quot;, &quot;c2&quot;)))</code> | <code>c1 c2 ⏎ r1  1  3 ⏎ r2  2  4</code> | <code>[,1] [,2] ⏎ [1,] 1 3 ⏎ [2,] 2 4</code> | `dimnames` is ignored (RC3). |
| Date class | Missing | <code>as.Date(&quot;2024-03-01&quot;) + 30</code> | <code>[1] &quot;2024-03-31&quot;</code> | <code>ERROR: object &#x27;as.Date&#x27; not found</code> |  |
| POSIXct date-time | Missing | <code>format(as.POSIXct(&quot;2024-03-01 12:00:00&quot;, tz = &quot;UTC&quot;), &quot;%H:%M&quot;)</code> | <code>[1] &quot;12:00&quot;</code> | <code>ERROR: object &#x27;format&#x27; not found</code> |  |

### Printing

16 checked: 3 done, 11 partial, 2 missing.

| Feature | Status | Snippet | R 4.6.0 | R-ust | Notes |
| --- | --- | --- | --- | --- | --- |
| Long vector wraps with index prefix | Partial | <code>1:30</code> | <code>[1]  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 1 …</code> | <code>[1] 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 1 …</code> | RC8. |
| Default 7 significant digits | Partial | <code>1 / 3 # default digits</code> | <code>[1] 0.3333333</code> | <code>[1] 0.333333333333</code> | RC8. |
| Named vector printing | Partial | <code>c(a = 1, b = 2)</code> | <code>a b  ⏎ 1 2</code> | <code>[1] 1 2</code> | RC3 and RC8. |
| Character vector alignment | Partial | <code>c(&quot;a&quot;, &quot;bbb&quot;)</code> | <code>[1] &quot;a&quot;   &quot;bbb&quot;</code> | <code>[1] &quot;a&quot; &quot;bbb&quot;</code> |  |
| Nested list printing | Partial | <code>list(a = 1, b = list(c = &quot;x&quot;))</code> | <code>$a ⏎ [1] 1 ⏎  ⏎ $b ⏎ $b$c ⏎ [1] &quot;x&quot; ⏎  ⏎ </code> | <code>$a ⏎ [1] 1 ⏎  ⏎ $b ⏎ $c ⏎ [1] &quot;x&quot;</code> | Nested element prefixes such as `$b$c` are not printed. |
| Data frame printing | Partial | <code>data.frame(x = 1:2, y = c(&quot;a&quot;, &quot;b&quot;))</code> | <code>x y ⏎ 1 1 a ⏎ 2 2 b</code> | <code>x y ⏎ 1 1 &quot;a&quot; ⏎ 2 2 &quot;b&quot;</code> | Strings print with quotes (RC8). |
| Factor printing | Done | <code>factor(c(&quot;b&quot;, &quot;a&quot;))</code> | <code>[1] b a ⏎ Levels: a b</code> | <code>[1] b a ⏎ Levels: a b</code> |  |
| Table printing | Partial | <code>table(c(&quot;a&quot;, &quot;b&quot;, &quot;a&quot;))</code> | <code> ⏎ a b  ⏎ 2 1</code> | <code>$a ⏎ [1] 2 ⏎  ⏎ $b ⏎ [1] 1</code> | Prints a named list (RC3). |
| print() inside a loop | Partial | <code>for (i in 1:2) print(i)</code> | <code>[1] 1 ⏎ [1] 2</code> | <code></code> | RC15. |
| print() inside a function | Partial | <code>f &lt;- function() { print(1); 2 }; f()</code> | <code>[1] 1 ⏎ [1] 2</code> | <code>[1] 2</code> | RC15. |
| cat() | Missing | <code>cat(&quot;a&quot;, 1, TRUE, &quot; ⏎ &quot;)</code> | <code>a 1 TRUE</code> | <code>ERROR: object &#x27;cat&#x27; not found</code> |  |
| print(quote = FALSE) | Partial | <code>print(c(&quot;a&quot;, &quot;b&quot;), quote = FALSE)</code> | <code>[1] a b</code> | <code>[1] &quot;a&quot; &quot;b&quot;</code> | `quote` is ignored. |
| format() / sprintf() / formatC() | Missing | <code>sprintf(&quot;%5.2f&#124;%s&quot;, 3.14159, &quot;x&quot;)</code> | <code>[1] &quot; 3.14&#124;x&quot;</code> | <code>ERROR: object &#x27;sprintf&#x27; not found</code> |  |
| Empty list printing | Done | <code>list()</code> | <code>list()</code> | <code>list()</code> |  |
| NULL printing | Partial | <code>NULL</code> | <code>NULL</code> | <code></code> | Prints nothing (RC8). |
| Negative zero prints as 0 | Done | <code>sum()</code> | <code>[1] 0</code> | <code>[1] 0</code> |  |

## Runtime and infrastructure

| Component | Status | Notes |
| --- | --- | --- |
| Parser | Partial | Handles the constructs marked Done above. Missing: hex, complex and raw string literals, backticks, `::`, `:::`, `@`, `~`, `\|>`, `\(x)`, `%op%` operators other than `%in%`, empty arguments, quoted assignment targets, and invalid L qualifier warnings. |
| Evaluator | Partial | Lazy promises, closures, `...`, exact and positional matching, and control flow work. Missing: environments as objects, reference semantics for enclosing environments, S3, S4 and reference class dispatch, condition system, `on.exit`, `sys.*` frame access, bytecode. |
| Object model (`r_ust_sexp`) | Partial | SEXPTYPE codes match R 4.6.0. Runtime values: NULL, logical, integer, double, character, list, factor, data frame, matrix, closure, promise. Missing: complex, raw, environment, symbol and language objects at R level, expression vectors, S4 objects, external pointers, weak references, general attributes. |
| Memory management (`r_ust_gc`) | Partial | A handle model and protect stack exist; evaluator values are plain Rust values with no R heap, generational collector, `gc()`, or memory profiling. |
| Printing | Partial | See RC8. |
| Interactive console (REPL) | Missing | `r_ust_cli` with no expression or file reports that the console is not implemented. |
| Command line front end | Partial | `r_ust_cli` accepts `-e`, `-f`/`--file`, a script path, `-q`/`--quiet`, `--vanilla` (no effect), `--version` and `--args` (stops option parsing). Missing: `Rscript`, `R CMD`, `--no-echo`, `--save`/`--no-save`, `--restore`, `commandArgs()`, other R options. |
| Startup files and environment | Missing | `.Rprofile`, `Rprofile.site`, `.Renviron`, `R_LIBS` and site library handling. |
| `options()` and `getOption()` | Missing |  |
| Locale, encodings, `Encoding()` | Missing | Strings are Rust UTF-8 strings; `\u` escapes are not decoded. |
| Condition system | Missing | `stop`, `warning`, `message`, `tryCatch`, `try`, `withCallingHandlers`, restarts. Errors abort evaluation with a message. |
| S3 dispatch | Missing | `UseMethod`, `NextMethod`, group generics, `class<-`, `inherits`, `structure`. |
| S4 (`methods`) | Missing | `setClass`, `new`, `setGeneric`, `setMethod`, `@`, `isVirtualClass`, `show`. |
| Reference classes | Missing | `setRefClass`. |
| Bytecode compiler (`compiler`) | Missing |  |
| Package system | Missing | `library`, `require`, namespaces, `loadNamespace`, `::`, lazy loading, `install.packages`, `R CMD INSTALL`/`check`/`build`. |
| Serialization | Missing | `saveRDS`, `readRDS`, `save`, `load`, `serialize`, workspace images. |
| Connections and file I/O | Missing | `file`, `url`, `gzfile`, `readLines`, `writeLines`, `scan`, `read.table`, `sink`, `source`. |
| System interface | Missing | `Sys.time`, `Sys.getenv`, `system`, `file.exists`, `proc.time`, `Sys.sleep`. |
| Dates and times | Missing | `Date`, `POSIXct`, `POSIXlt`, `difftime`, `strptime`, `format`. |
| C API (`Rinternals.h`, `.Call`, `.C`, `.External`) | Missing | No R object API for compiled code; packages with compiled code cannot load. |
| Rmath standalone C API | Partial | All 154 symbols exported from `r_ust_ffi`; accuracy caveats in the next section. |
| Graphics engine and devices | Missing | `graphics`, `grDevices`, `grid`, PDF, PNG, screen devices. |
| Tcl/Tk (`tcltk`) | Missing |  |
| Parallel (`parallel`) | Missing |  |
| Help system | Missing | `help`, `?`, Rd rendering, `example`. |
| Debugging | Missing | `browser`, `debug`, `traceback`, `trace`, `Rprof`. |
| Datasets (`datasets`) | Missing | No data objects are defined. |

## Rmath C library

`r_ust_nmath` implements and `r_ust_ffi` exports all 154 public functions of R's standalone `Rmath.h`. The evidence below comes from R level calls; no C level comparison against R's own `libRmath` was run.

154 functions: 67 done, 41 partial, 46 unverified.

| Function | Status | Notes |
| --- | --- | --- |
| `R_finite` | Unverified |  |
| `R_isnancpp` | Unverified |  |
| `R_pow` | Unverified |  |
| `R_pow_di` | Unverified |  |
| `R_unif_index` | Unverified |  |
| `Rlog1p` | Unverified |  |
| `Rtanpi` | Unverified |  |
| `bessel_i` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_i_ex` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_j` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_j_ex` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_k` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_k_ex` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_y` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `bessel_y_ex` | Partial | Series, reflection and asymptotic approximations instead of R's Cody and Amos routines. |
| `beta` | Done | Matches R through `beta()`. |
| `choose` | Done | Matches R through `choose()`. |
| `cospi` | Unverified |  |
| `dbeta` | Done | Matches R through `dbeta()`. |
| `dbinom` | Done | Matches R through `dbinom()`. |
| `dbinom_raw` | Unverified |  |
| `dcauchy` | Done | Matches R through `dcauchy()`. |
| `dchisq` | Done | Matches R through `dchisq()`. |
| `dexp` | Done | Matches R through `dexp()`. |
| `df` | Done | Matches R through `df()`. |
| `dgamma` | Done | Matches R when called with `scale`; the R level wrapper has RC6. |
| `dgeom` | Done | Matches R through `dgeom()`. |
| `dhyper` | Done | Matches R through `dhyper()`. |
| `digamma` | Done | Matches R through `digamma()`. |
| `dlnorm` | Done | Matches R through `dlnorm()`. |
| `dlogis` | Done | Matches R through `dlogis()`. |
| `dnbeta` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `dnbinom` | Done | Matches R through `dnbinom()`. |
| `dnbinom_mu` | Unverified |  |
| `dnchisq` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `dnf` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `dnorm` | Done | Matches R through `dnorm()`. |
| `dnt` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `dpois` | Done | Matches R through `dpois()`. |
| `dpois_raw` | Unverified |  |
| `dpsifn` | Partial | Polygamma accuracy about 1e-5 where checked (RC13). |
| `dsignrank` | Done | Matches R through `dsignrank()`. |
| `dt` | Done | Matches R through `dt()`. |
| `dunif` | Done | Matches R through `dunif()`. |
| `dweibull` | Done | Matches R through `dweibull()`. |
| `dwilcox` | Done | Matches R through `dwilcox()`. |
| `exp_rand` | Unverified |  |
| `fmax2` | Unverified |  |
| `fmin2` | Unverified |  |
| `fprec` | Unverified |  |
| `fround` | Unverified |  |
| `fsign` | Unverified |  |
| `ftrunc` | Unverified |  |
| `gammafn` | Done | Matches R through `gamma()`. |
| `get_seed` | Unverified |  |
| `imax2` | Unverified |  |
| `imin2` | Unverified |  |
| `lbeta` | Done | Matches R through `lbeta()`. |
| `lchoose` | Done | Matches R through `lchoose()`. |
| `lgamma1p` | Unverified |  |
| `lgammafn` | Done | Matches R through `lgamma()`. |
| `lgammafn_sign` | Unverified |  |
| `log1mexp` | Unverified |  |
| `log1pexp` | Unverified |  |
| `log1pmx` | Unverified |  |
| `logspace_add` | Unverified |  |
| `logspace_sub` | Unverified |  |
| `logspace_sum` | Unverified |  |
| `norm_rand` | Unverified |  |
| `pbeta` | Done | Matches R through `pbeta()`. |
| `pbinom` | Done | Matches R through `pbinom()`. |
| `pcauchy` | Done | Matches R through `pcauchy()`. |
| `pchisq` | Done | Matches R through `pchisq()`. |
| `pentagamma` | Partial | Polygamma accuracy about 1e-5 where checked (RC13). |
| `pexp` | Done | Matches R through `pexp()`. |
| `pf` | Done | Matches R through `pf()`. |
| `pgamma` | Done | Matches R when called with `scale`; the R level wrapper has RC6. |
| `pgeom` | Done | Matches R through `pgeom()`. |
| `phyper` | Done | Matches R through `phyper()`. |
| `plnorm` | Done | Matches R through `plnorm()`. |
| `plogis` | Done | Matches R through `plogis()`. |
| `pnbeta` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `pnbinom` | Done | Matches R through `pnbinom()`. |
| `pnbinom_mu` | Unverified |  |
| `pnchisq` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `pnf` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `pnorm` | Done | Matches R through `pnorm()`. |
| `pnorm_both` | Unverified |  |
| `pnt` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `pow1p` | Unverified |  |
| `ppois` | Done | Matches R through `ppois()`. |
| `psigamma` | Partial | Polygamma accuracy about 1e-5 where checked (RC13). |
| `psignrank` | Done | Matches R through `psignrank()`. |
| `pt` | Done | Matches R through `pt()`. |
| `ptukey` | Partial | Monotone approximation instead of R's quadrature. |
| `punif` | Done | Matches R through `punif()`. |
| `pweibull` | Done | Matches R through `pweibull()`. |
| `pwilcox` | Done | Matches R through `pwilcox()`. |
| `qbeta` | Done | Matches R through `qbeta()`. |
| `qbinom` | Done | Matches R through `qbinom()`. |
| `qcauchy` | Done | Matches R through `qcauchy()`. |
| `qchisq` | Done | Matches R through `qchisq()`. |
| `qexp` | Done | Matches R through `qexp()`. |
| `qf` | Done | Matches R through `qf()`. |
| `qgamma` | Done | Matches R when called with `scale`; the R level wrapper has RC6. |
| `qgeom` | Done | Matches R through `qgeom()`. |
| `qhyper` | Done | Matches R through `qhyper()`. |
| `qlnorm` | Done | Matches R through `qlnorm()`. |
| `qlogis` | Done | Matches R through `qlogis()`. |
| `qnbeta` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `qnbinom` | Done | Matches R through `qnbinom()`. |
| `qnbinom_mu` | Unverified |  |
| `qnchisq` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `qnf` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `qnorm` | Done | Matches R through `qnorm()`. |
| `qnt` | Partial | Mixture, integration and bisection methods; tail agreement with R not validated. |
| `qpois` | Done | Matches R through `qpois()`. |
| `qsignrank` | Done | Matches R through `qsignrank()`. |
| `qt` | Done | Matches R through `qt()`. |
| `qtukey` | Partial | Monotone approximation instead of R's quadrature. |
| `qunif` | Done | Matches R through `qunif()`. |
| `qweibull` | Done | Matches R through `qweibull()`. |
| `qwilcox` | Done | Matches R through `qwilcox()`. |
| `rbeta` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rbinom` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rcauchy` | Unverified | Stream comparison blocked by RC5. |
| `rchisq` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rexp` | Unverified | Stream comparison blocked by RC5. |
| `rf` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rgamma` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rgeom` | Unverified | Stream comparison blocked by RC5. |
| `rhyper` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rlnorm` | Unverified | Stream comparison blocked by RC5. |
| `rlogis` | Unverified | Stream comparison blocked by RC5. |
| `rmultinom` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rnbinom` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rnbinom_mu` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rnchisq` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rnorm` | Unverified | Stream comparison blocked by RC5. |
| `rpois` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rsignrank` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `rt` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `runif` | Unverified | Stream comparison blocked by RC5. |
| `rweibull` | Unverified | Stream comparison blocked by RC5. |
| `rwilcox` | Partial | Generator algorithm differs from R; draws are not R's stream. |
| `set_seed` | Unverified |  |
| `sign` | Done | Matches R through `sign()`. |
| `signrank_free` | Done | No-op; exact tables need no cache. |
| `sinpi` | Unverified |  |
| `tanpi` | Unverified |  |
| `tetragamma` | Partial | Polygamma accuracy about 1e-5 where checked (RC13). |
| `trigamma` | Partial | Polygamma accuracy about 1e-5 where checked (RC13). |
| `unif_rand` | Unverified |  |
| `wilcox_free` | Done | No-op; exact tables need no cache. |

## Functions by package

Each package lists the available objects with their status and evidence, then the missing objects. Evidence shows the first call whose values differ from R.

### base

1407 objects: 155 done, 65 partial, 1187 missing.

| Name | Kind | Status | Notes | Evidence |
| --- | --- | --- | --- | --- |
| `-` | builtin | Partial | Integer values and overflow are implemented; warnings are missing (RC4). |  |
| `:` | builtin | Partial | Integer storage implemented; compact long-vector storage is missing (RC4). |  |
| `!` | builtin | Done |  |  |
| `!=` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `(` | builtin | Partial | A parenthesized assignment does not print (RC12). |  |
| `[` | special | Partial | Character indexing only for lists; `drop` and empty indices with named arguments are parse errors. |  |
| `[.data.frame` | closure | Partial | Row and column indexing work; row names are renumbered (RC3). |  |
| `[.factor` | closure | Done |  |  |
| `[[` | special | Partial | No recursive indexing with a vector index. |  |
| `[[.data.frame` | closure | Done |  |  |
| `[[<-` | special | Done |  |  |
| `[[<-.data.frame` | closure | Done |  |  |
| `[<-` | special | Partial | Growing a double vector pads with `NA` (RC2); general replacement semantics remain incomplete. |  |
| `[<-.data.frame` | closure | Done |  |  |
| `{` | special | Done |  |  |
| `*` | builtin | Partial | Integer values and overflow are implemented; warnings are missing (RC4). |  |
| `/` | builtin | Done |  |  |
| `&` | builtin | Done |  |  |
| `&&` | special | Done |  |  |
| `%in%` | closure | Done |  |  |
| `^` | builtin | Done | Core precedence fixed (RC11). |  |
| `+` | builtin | Partial | Integer values and overflow are implemented; warnings are missing (RC4). |  |
| `<` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `<-` | special | Done |  |  |
| `<<-` | special | Partial | Assigns in the global environment instead of the nearest enclosing function environment. |  |
| `<=` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `=` | special | Done |  |  |
| `==` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `>` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `>=` | builtin | Partial | Numeric comparison fixed; broader gaps remain (RC1). |  |
| `\|` | builtin | Done |  |  |
| `\|\|` | special | Done |  |  |
| `$` | special | Partial | No partial name matching. |  |
| `$<-` | special | Done |  |  |
| `$<-.data.frame` | closure | Done |  |  |
| `abs` | builtin | Done | Values match; printing differs (RC8). |  |
| `acos` | builtin | Done |  |  |
| `acosh` | builtin | Done |  |  |
| `all` | builtin | Done |  |  |
| `any` | builtin | Done |  |  |
| `anyDuplicated` | closure | Partial | Arguments not referenced: `incomparables`. |  |
| `append` | closure | Done | Values match; printing differs (RC8). |  |
| `apply` | closure | Done | Values match; printing differs (RC8). |  |
| `as.character` | builtin | Partial | Large numbers convert without scientific notation (RC14). | <code>as.character(c(1.5, 2, 1e6, 123456789))</code>: R <code>[1] &quot;1.5&quot;       &quot;2&quot;         &quot;1e+06&quot;     &quot;123456789&quot;</code>, R-ust <code>[1] &quot;1.5&quot; &quot;2&quot; &quot;1000000&quot; &quot;123456789&quot;</code> |
| `as.double` | builtin | Done | Values match; printing differs (RC8). |  |
| `as.factor` | closure | Done |  |  |
| `as.integer` | builtin | Done | Values match; printing differs (RC8). |  |
| `as.list` | closure | Done | Values match; printing differs (RC8). |  |
| `as.logical` | builtin | Done | Values match; printing differs (RC8). |  |
| `as.numeric` | builtin | Done | Values match; printing differs (RC8). |  |
| `asin` | builtin | Done |  |  |
| `asinh` | builtin | Done | Values match; printing differs (RC8). |  |
| `atan` | builtin | Done | Values match; printing differs (RC8). |  |
| `atan2` | closure | Done | Values match; printing differs (RC8). |  |
| `atanh` | builtin | Done |  |  |
| `beta` | closure | Done | Values match; printing differs (RC8). |  |
| `break` | special | Done |  |  |
| `c` | builtin | Partial | Arguments not referenced: `recursive`, `use.names`. Element names are dropped (RC3). | <code>c(a = 1, b = 2, 3)</code>: R <code>a b    ⏎ 1 2 3</code>, R-ust <code>[1] 1 2 3</code> |
| `cbind` | closure | Done | Values match; printing differs (RC8). |  |
| `ceiling` | builtin | Done |  |  |
| `chartr` | closure | Done |  |  |
| `choose` | closure | Done | Values match; printing differs (RC8). |  |
| `class` | builtin | Done |  |  |
| `col` | closure | Done | Values match; printing differs (RC8). |  |
| `colMeans` | closure | Partial | Arguments not referenced: `dims`. |  |
| `colSums` | closure | Partial | Arguments not referenced: `dims`. Values match; printing differs (RC8). |  |
| `cos` | builtin | Done | Values match; printing differs (RC8). |  |
| `cosh` | builtin | Done | Values match; printing differs (RC8). |  |
| `crossprod` | builtin | Done | Values match; printing differs (RC8). |  |
| `cummax` | builtin | Done | Values match; printing differs (RC8). |  |
| `cummin` | builtin | Done | Values match; printing differs (RC8). |  |
| `cumprod` | builtin | Done | Values match; printing differs (RC8). |  |
| `cumsum` | builtin | Done | Values match; printing differs (RC8). |  |
| `data.frame` | closure | Partial | Arguments not referenced: `check.rows`, `check.names`, `fix.empty.names`, `stringsAsFactors`. Character columns print with quotes (RC8). | <code>data.frame(x = 1:2, y = c(&quot;a&quot;, &quot;b&quot;))</code>: R <code>x y ⏎ 1 1 a ⏎ 2 2 b</code>, R-ust <code>x y ⏎ 1 1 &quot;a&quot; ⏎ 2 2 &quot;b&quot;</code> |
| `diag` | closure | Partial | Arguments not referenced: `names`. Values match; printing differs (RC8). |  |
| `diff` | closure | Done | Values match; printing differs (RC8). |  |
| `digamma` | builtin | Done | Values match; printing differs (RC8). |  |
| `dim` | builtin | Done |  |  |
| `duplicated` | closure | Partial | Arguments not referenced: `incomparables`, `fromLast`, `nmax`. Values match; printing differs (RC8). |  |
| `endsWith` | closure | Done | Values match; printing differs (RC8). |  |
| `exp` | builtin | Done | Values match; printing differs (RC8). |  |
| `expm1` | builtin | Done | Values match; printing differs (RC8). |  |
| `F` | logical | Done |  |  |
| `factor` | closure | Partial | Arguments not referenced: `exclude`, `nmax`. |  |
| `factorial` | closure | Done |  |  |
| `floor` | builtin | Done | Values match; printing differs (RC8). |  |
| `for` | special | Done |  |  |
| `function` | special | Partial | No reference semantics for enclosing environments; printing shows only the header. |  |
| `gamma` | builtin | Done | Values match; printing differs (RC8). |  |
| `grep` | closure | Partial | Arguments not referenced: `perl`, `useBytes`. Bracket expressions fail (RC7); no match prints `[1]` (RC8). | <code>grep(&quot;^[a-c]&quot;, c(&quot;apple&quot;, &quot;dog&quot;, &quot;cat&quot;))</code>: R <code>[1] 1 3</code>, R-ust <code>[1]</code> |
| `grepl` | closure | Partial | Arguments not referenced: `perl`, `useBytes`. Bracket expressions fail (RC7). | <code>grepl(&quot;[0-9]&quot;, c(&quot;a1&quot;, &quot;b&quot;))</code>: R <code>[1]  TRUE FALSE</code>, R-ust <code>[1] FALSE FALSE</code> |
| `gsub` | closure | Partial | Arguments not referenced: `perl`, `useBytes`. Bracket expressions fail (RC7). | <code>gsub(&quot;[aeiou]&quot;, &quot;&quot;, &quot;banana&quot;)</code>: R <code>[1] &quot;bnn&quot;</code>, R-ust <code>[1] &quot;banana&quot;</code> |
| `identical` | closure | Partial | Arguments not referenced: `num.eq`, `single.NA`, `attrib.as.set`, `ignore.bytecode`, `ignore.environment`, `ignore.srcref`, `extptr.as.ref`. |  |
| `if` | special | Done |  |  |
| `ifelse` | closure | Done | Values match; printing differs (RC8). |  |
| `intersect` | closure | Done |  |  |
| `is.atomic` | builtin | Done |  |  |
| `is.character` | builtin | Done |  |  |
| `is.data.frame` | closure | Done |  |  |
| `is.double` | builtin | Done |  |  |
| `is.factor` | closure | Done |  |  |
| `is.finite` | builtin | Done | Values match; printing differs (RC8). |  |
| `is.function` | builtin | Done |  |  |
| `is.infinite` | builtin | Done | Values match; printing differs (RC8). |  |
| `is.integer` | builtin | Done |  |  |
| `is.list` | builtin | Done |  |  |
| `is.logical` | builtin | Done |  |  |
| `is.matrix` | builtin | Done |  |  |
| `is.na` | builtin | Done | Values match; printing differs (RC8). |  |
| `is.nan` | builtin | Done | Values match; printing differs (RC8). |  |
| `is.null` | builtin | Done |  |  |
| `is.numeric` | builtin | Done |  |  |
| `is.ordered` | closure | Done |  |  |
| `is.unsorted` | closure | Done |  |  |
| `is.vector` | closure | Partial | Arguments not referenced: `mode`. |  |
| `isFALSE` | closure | Done |  |  |
| `isTRUE` | closure | Done |  |  |
| `lapply` | closure | Done | Values match; printing differs (RC8). |  |
| `lbeta` | closure | Done | Values match; printing differs (RC8). |  |
| `lchoose` | closure | Done | Values match; printing differs (RC8). |  |
| `length` | builtin | Done |  |  |
| `lengths` | closure | Partial | Names are dropped (RC3). | <code>lengths(list(a = 1:3, b = 1))</code>: R <code>a b  ⏎ 3 1</code>, R-ust <code>[1] 3 1</code> |
| `levels` | closure | Done |  |  |
| `lfactorial` | closure | Done | Values match; printing differs (RC8). |  |
| `lgamma` | builtin | Done |  |  |
| `list` | builtin | Done | Values match; printing differs (RC8). |  |
| `log` | special | Done | Values match; printing differs (RC8). |  |
| `log10` | builtin | Done |  |  |
| `log1p` | builtin | Done | Values match; printing differs (RC8). |  |
| `log2` | builtin | Done | Values match; printing differs (RC8). |  |
| `logb` | closure | Done | Values match; printing differs (RC8). |  |
| `lower.tri` | closure | Done | Values match; printing differs (RC8). |  |
| `match` | closure | Partial | Arguments not referenced: `incomparables`. Values match; printing differs (RC8). |  |
| `matrix` | closure | Partial | Arguments not referenced: `dimnames`. Values match; printing differs (RC8). |  |
| `max` | builtin | Done |  |  |
| `mean` | closure | Partial | Arguments not referenced: `trim`. |  |
| `min` | builtin | Done |  |  |
| `missing` | special | Done |  |  |
| `mode` | closure | Done |  |  |
| `names` | builtin | Partial | Returns nothing for named atomic vectors (RC3). | <code>names(c(a = 1, b = 2))</code>: R <code>[1] &quot;a&quot; &quot;b&quot;</code>, R-ust <code></code> |
| `names<-` | builtin | Partial | Works for lists and data frames only (RC3). |  |
| `nchar` | closure | Partial | Arguments not referenced: `type`, `allowNA`, `keepNA`. Values match; printing differs (RC8). |  |
| `ncol` | closure | Done |  |  |
| `next` | special | Done |  |  |
| `nlevels` | closure | Done |  |  |
| `nrow` | closure | Done |  |  |
| `nzchar` | builtin | Done | Values match; printing differs (RC8). |  |
| `order` | closure | Partial | Arguments not referenced: `method`. |  |
| `ordered` | closure | Done |  |  |
| `paste` | closure | Partial | Arguments not referenced: `recycle0`. |  |
| `paste0` | closure | Partial | Arguments not referenced: `recycle0`. |  |
| `pmax` | closure | Done | Values match; printing differs (RC8). |  |
| `pmin` | closure | Done |  |  |
| `print` | closure | Partial | Arguments not referenced: `digits`, `quote`, `na.print`, `print.gap`, `right`, `max`, `width`, `useSource`. Returns its argument without printing, so it shows nothing inside loops or functions (RC15); formatting options are ignored (RC8). | <code>for (i in 1:2) print(i)</code>: R <code>[1] 1 ⏎ [1] 2</code>, R-ust <code></code> |
| `prod` | builtin | Done |  |  |
| `psigamma` | closure | Partial | Accuracy about 1e-5 (RC13). | <code>psigamma(c(1.5, 2), 2)</code>: R <code>[1] -0.828796644234 -0.404113806319</code>, R-ust <code>[1] -0.828777839174 -0.404091749207</code> |
| `range` | builtin | Done | Values match; printing differs (RC8). |  |
| `rank` | closure | Done | Values match; printing differs (RC8). |  |
| `rbind` | closure | Done | Values match; printing differs (RC8). |  |
| `rep` | special | Partial | Arguments not referenced: `length.out`, `each`. `each` is silently ignored (RC9). | <code>rep(1:2, times = 3, each = 2)</code>: R <code>[1] 1 1 2 2 1 1 2 2 1 1 2 2</code>, R-ust <code>[1] 1 2 1 2 1 2</code> |
| `rep_len` | closure | Done |  |  |
| `rep.int` | closure | Done |  |  |
| `repeat` | special | Done |  |  |
| `replace` | closure | Done |  |  |
| `return` | special | Done |  |  |
| `rev` | closure | Done | Values match; printing differs (RC8). |  |
| `round` | special | Done | Values match; printing differs (RC8). |  |
| `row` | closure | Done | Values match; printing differs (RC8). |  |
| `rowMeans` | closure | Partial | Arguments not referenced: `dims`. |  |
| `rowSums` | closure | Partial | Arguments not referenced: `dims`. Values match; printing differs (RC8). |  |
| `sample` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); sample(10, 4)</code>: R <code>[1]  1  5 10  8</code>, R-ust <code>[1] 2 4 5 8</code> |
| `sample.int` | closure | Partial | Arguments not referenced: `useHash`. Draws differ from R (RC5). | <code>set.seed(42); sample.int(100, 3)</code>: R <code>[1] 49 65 25</code>, R-ust <code>[1] 59 99 69</code> |
| `sapply` | closure | Partial | Arguments not referenced: `USE.NAMES`. `USE.NAMES` is accepted but ignored; result names are dropped (RC3). | <code>sapply(c(a = 1, b = 2), function(i) i^2)</code>: R <code>a b  ⏎ 1 4</code>, R-ust <code>[1] 1 4</code> |
| `seq` | closure | Partial | Arguments not referenced: `length.out`, `along.with`. Values match; printing differs (RC8). |  |
| `seq_along` | builtin | Done |  |  |
| `seq_len` | builtin | Done |  |  |
| `seq.int` | builtin | Partial | Arguments not referenced: `length.out`, `along.with`. Values match; printing differs (RC8). |  |
| `set.seed` | closure | Partial | Arguments not referenced: `kind`, `normal.kind`, `sample.kind`. Draws differ from R (RC5). | <code>set.seed(1); runif(2)</code>: R <code>[1] 0.265508663142 0.372123899637</code>, R-ust <code>[1] 0.564111970729 0.293241983814</code> |
| `setdiff` | closure | Done | Values match; printing differs (RC8). |  |
| `setequal` | closure | Done |  |  |
| `sign` | builtin | Done | Values match; printing differs (RC8). |  |
| `signif` | special | Done | Values match; printing differs (RC8). |  |
| `sin` | builtin | Done | Values match; printing differs (RC8). |  |
| `sinh` | builtin | Done | Values match; printing differs (RC8). |  |
| `sort` | closure | Partial | Arguments not referenced: `na.last`. Values match; printing differs (RC8). |  |
| `split` | closure | Partial | Arguments not referenced: `sep`, `lex.order`. Group names print as `$1` instead of R's backtick form (RC8). | <code>split(c(10, 9, 100, 2), c(1, 2, 1, 2))</code>: R <code>$`1` ⏎ [1]  10 100 ⏎  ⏎ $`2` ⏎ [1] 9 2 ⏎ </code>, R-ust <code>$1 ⏎ [1] 10 100 ⏎  ⏎ $2 ⏎ [1] 9 2</code> |
| `sqrt` | builtin | Done | Values match; printing differs (RC8). |  |
| `startsWith` | closure | Done | Values match; printing differs (RC8). |  |
| `strrep` | closure | Done |  |  |
| `strsplit` | closure | Partial | Arguments not referenced: `perl`, `useBytes`. Bracket expressions fail (RC7). | <code>strsplit(&quot;a1b2c&quot;, &quot;[0-9]&quot;)</code>: R <code>[[1]] ⏎ [1] &quot;a&quot; &quot;b&quot; &quot;c&quot; ⏎ </code>, R-ust <code>[[1]] ⏎ [1] &quot;a1b2c&quot;</code> |
| `strtrim` | closure | Done |  |  |
| `sub` | closure | Partial | Arguments not referenced: `perl`, `useBytes`. Bracket expressions fail (RC7). | <code>sub(&quot;[0-9]+&quot;, &quot;#&quot;, &quot;a12b&quot;)</code>: R <code>[1] &quot;a#b&quot;</code>, R-ust <code>[1] &quot;a12b&quot;</code> |
| `subset` | closure | Partial | Row names are renumbered (RC3). | <code>subset(data.frame(x = 1:4, y = 4:1), x &gt; 2)</code>: R <code>x y ⏎ 3 3 2 ⏎ 4 4 1</code>, R-ust <code>x y ⏎ 1 3 2 ⏎ 2 4 1</code> |
| `substr` | closure | Done |  |  |
| `substring` | closure | Done |  |  |
| `sum` | builtin | Done |  |  |
| `t` | closure | Done | Values match; printing differs (RC8). |  |
| `T` | logical | Done |  |  |
| `table` | closure | Partial | Arguments not referenced: `dnn`, `deparse.level`. Returns a named list, not a `table` object (RC3). | <code>table(c(&quot;a&quot;, &quot;b&quot;, &quot;a&quot;, &quot;c&quot;))</code>: R <code> ⏎ a b c  ⏎ 2 1 1</code>, R-ust <code>$a ⏎ [1] 2 ⏎  ⏎ $b ⏎ [1] 1 ⏎  ⏎ $c ⏎ [1] 1</code> |
| `tabulate` | closure | Partial |  | <code>tabulate(c(10, 9, 100, 2))</code>: R <code>[1] 0 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 …</code>, R-ust <code>[1] 0 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 …</code> |
| `tan` | builtin | Done | Values match; printing differs (RC8). |  |
| `tanh` | builtin | Done | Values match; printing differs (RC8). |  |
| `tcrossprod` | builtin | Done | Values match; printing differs (RC8). |  |
| `tolower` | closure | Done |  |  |
| `toupper` | closure | Done |  |  |
| `transform` | closure | Done |  |  |
| `trigamma` | builtin | Partial | Accuracy about 1e-5 (RC13). | <code>trigamma(c(0.5, 0.25, 0.75))</code>: R <code>[1]  4.93480220054 17.19732915451  2.54187964767</code>, R-ust <code>[1] 4.93472378564 17.197245991656 2.541805626397</code> |
| `trimws` | closure | Partial | Arguments not referenced: `whitespace`. |  |
| `trunc` | builtin | Done |  |  |
| `typeof` | closure | Done |  |  |
| `union` | closure | Done | Values match; printing differs (RC8). |  |
| `unique` | closure | Partial | Arguments not referenced: `incomparables`, `fromLast`, `nmax`. Values match; printing differs (RC8). |  |
| `unlist` | closure | Partial | Arguments not referenced: `use.names`. Names are dropped (RC3). | <code>unlist(list(a = 1, b = list(c = 2, d = 3)))</code>: R <code>a b.c b.d  ⏎   1   2   3</code>, R-ust <code>[1] 1 2 3</code> |
| `upper.tri` | closure | Done | Values match; printing differs (RC8). |  |
| `which` | closure | Partial | Arguments not referenced: `arr.ind`, `useNames`. |  |
| `which.max` | closure | Done |  |  |
| `which.min` | closure | Done |  |  |
| `while` | special | Done |  |  |
| `with` | closure | Done |  |  |
| `xor` | closure | Done | Values match; printing differs (RC8). |  |

Missing (1184):

|   |   |   |   |
| --- | --- | --- | --- |
| `-.Date` | `-.POSIXt` | `::` | `:::` |
| `!.hexmode` | `!.octmode` | `...elt` | `...length` |
| `...names` | `..deparseOpts` | `..getNamespace` | `.amatch_bounds` |
| `.amatch_costs` | `.ArgsEnv` | `.AutoloadEnv` | `.BaseNamespaceEnv` |
| `.bincode` | `.C` | `.C_R_addTaskCallback` | `.C_R_getTaskCallbackNames` |
| `.C_R_removeTaskCallback` | `.cache_class` | `.Call` | `.Call.graphics` |
| `.check_tzones` | `.class2` | `.col` | `.colMeans` |
| `.colSums` | `.Date` | `.decode_numeric_version` | `.Defunct` |
| `.deparseOpts` | `.Deprecated` | `.detach` | `.Device` |
| `.Devices` | `.difftime` | `.doSortWrap` | `.doTrace` |
| `.doWrap` | `.dynLibs` | `.encode_numeric_version` | `.expand_R_libs_env_var` |
| `.External` | `.External.graphics` | `.External2` | `.F_dqrcf` |
| `.F_dqrdc2` | `.F_dqrqty` | `.F_dqrqy` | `.F_dqrrsd` |
| `.F_dqrxb` | `.F_dtrco` | `.First.sys` | `.format.zeros` |
| `.formula2varlist` | `.Fortran` | `.GenericArgsEnv` | `.getNamespace` |
| `.getNamespaceInfo` | `.getRequiredPackages` | `.getRequiredPackages2` | `.GlobalEnv` |
| `.gt` | `.gtn` | `.handleSimpleError` | `.Internal` |
| `.internalGenerics` | `.isMethodsDispatchOn` | `.isOpen` | `.kappa_tri` |
| `.knownS3Generics` | `.kronecker` | `.Last.value` | `.LC.categories` |
| `.leap.seconds` | `.libPaths` | `.Library` | `.Library.site` |
| `.Machine` | `.make_numeric_version` | `.makeMessage` | `.mapply` |
| `.maskedMsg` | `.mergeExportMethods` | `.mergeImportMethods` | `.noGenerics` |
| `.NotYetImplemented` | `.NotYetUsed` | `.Options` | `.OptRequireMethods` |
| `.packages` | `.Platform` | `.popath` | `.POSIXct` |
| `.POSIXlt` | `.pretty` | `.Primitive` | `.primTrace` |
| `.primUntrace` | `.rangeNum` | `.rmpkg` | `.row` |
| `.row_names_info` | `.rowMeans` | `.rowNamesDF<-` | `.rowSums` |
| `.S3_methods_table` | `.S3method` | `.S3PrimitiveGenerics` | `.Script` |
| `.set_ops_need_as_vector` | `.set_row_names` | `.signalSimpleWarning` | `.standard_regexps` |
| `.subset` | `.subset2` | `.sys.timezone` | `.TAOCP1997init` |
| `.traceback` | `.tryResumeInterrupt` | `.userHooksEnv` | `.valid.factor` |
| `[.AsIs` | `[.Date` | `[.difftime` | `[.Dlist` |
| `[.DLLInfoList` | `[.hexmode` | `[.listof` | `[.noquote` |
| `[.numeric_version` | `[.octmode` | `[.POSIXct` | `[.POSIXlt` |
| `[.simple.list` | `[.table` | `[.warnings` | `[[.Date` |
| `[[.factor` | `[[.numeric_version` | `[[.POSIXct` | `[[.POSIXlt` |
| `[[<-.factor` | `[[<-.numeric_version` | `[[<-.POSIXlt` | `[<-.Date` |
| `[<-.difftime` | `[<-.factor` | `[<-.numeric_version` | `[<-.POSIXct` |
| `[<-.POSIXlt` | `@` | `@<-` | `*.difftime` |
| `/.difftime` | `&.hexmode` | `&.octmode` | `%*%` |
| `%/%` | `%%` | `%\|\|%` | `%notin%` |
| `%o%` | `%x%` | `+.Date` | `+.POSIXt` |
| `\|.hexmode` | `\|.octmode` | `~` | `$.DLLInfo` |
| `$.package_version` | `$<-.POSIXlt` | `abbreviate` | `activeBindingFunction` |
| `addNA` | `addTaskCallback` | `agrep` | `agrepl` |
| `alist` | `all.equal` | `all.equal.character` | `all.equal.default` |
| `all.equal.environment` | `all.equal.envRefClass` | `all.equal.factor` | `all.equal.formula` |
| `all.equal.function` | `all.equal.language` | `all.equal.list` | `all.equal.numeric` |
| `all.equal.POSIXt` | `all.equal.raw` | `all.names` | `all.vars` |
| `allowInterrupts` | `anyDuplicated.array` | `anyDuplicated.data.frame` | `anyDuplicated.default` |
| `anyDuplicated.matrix` | `anyNA` | `anyNA.data.frame` | `anyNA.numeric_version` |
| `anyNA.POSIXlt` | `aperm` | `aperm.default` | `aperm.table` |
| `Arg` | `args` | `array` | `array2DF` |
| `arrayInd` | `as.array` | `as.array.default` | `as.call` |
| `as.character.condition` | `as.character.Date` | `as.character.default` | `as.character.error` |
| `as.character.factor` | `as.character.hexmode` | `as.character.numeric_version` | `as.character.octmode` |
| `as.character.POSIXt` | `as.character.srcref` | `as.complex` | `as.data.frame` |
| `as.data.frame.array` | `as.data.frame.AsIs` | `as.data.frame.character` | `as.data.frame.complex` |
| `as.data.frame.data.frame` | `as.data.frame.Date` | `as.data.frame.default` | `as.data.frame.difftime` |
| `as.data.frame.factor` | `as.data.frame.integer` | `as.data.frame.list` | `as.data.frame.logical` |
| `as.data.frame.matrix` | `as.data.frame.model.matrix` | `as.data.frame.noquote` | `as.data.frame.numeric` |
| `as.data.frame.numeric_version` | `as.data.frame.ordered` | `as.data.frame.POSIXct` | `as.data.frame.POSIXlt` |
| `as.data.frame.raw` | `as.data.frame.table` | `as.data.frame.ts` | `as.data.frame.vector` |
| `as.Date` | `as.Date.character` | `as.Date.default` | `as.Date.factor` |
| `as.Date.numeric` | `as.Date.POSIXct` | `as.Date.POSIXlt` | `as.difftime` |
| `as.double.difftime` | `as.double.POSIXlt` | `as.environment` | `as.expression` |
| `as.expression.default` | `as.function` | `as.function.default` | `as.hexmode` |
| `as.list.data.frame` | `as.list.Date` | `as.list.default` | `as.list.difftime` |
| `as.list.environment` | `as.list.factor` | `as.list.function` | `as.list.numeric_version` |
| `as.list.POSIXct` | `as.list.POSIXlt` | `as.logical.factor` | `as.matrix` |
| `as.matrix.data.frame` | `as.matrix.default` | `as.matrix.noquote` | `as.matrix.POSIXlt` |
| `as.name` | `as.null` | `as.null.default` | `as.numeric_version` |
| `as.octmode` | `as.ordered` | `as.package_version` | `as.pairlist` |
| `as.POSIXct` | `as.POSIXct.Date` | `as.POSIXct.default` | `as.POSIXct.numeric` |
| `as.POSIXct.POSIXlt` | `as.POSIXlt` | `as.POSIXlt.character` | `as.POSIXlt.Date` |
| `as.POSIXlt.default` | `as.POSIXlt.factor` | `as.POSIXlt.numeric` | `as.POSIXlt.POSIXct` |
| `as.qr` | `as.raw` | `as.single` | `as.single.default` |
| `as.symbol` | `as.table` | `as.table.default` | `as.vector` |
| `as.vector.data.frame` | `as.vector.factor` | `as.vector.POSIXlt` | `asNamespace` |
| `asplit` | `asS3` | `asS4` | `assign` |
| `attach` | `attachNamespace` | `attr` | `attr.all.equal` |
| `attr<-` | `attributes` | `attributes<-` | `autoload` |
| `autoloader` | `backsolve` | `balancePOSIXlt` | `baseenv` |
| `basename` | `besselI` | `besselJ` | `besselK` |
| `besselY` | `bindingIsActive` | `bindingIsLocked` | `bindtextdomain` |
| `bitwAnd` | `bitwNot` | `bitwOr` | `bitwShiftL` |
| `bitwShiftR` | `bitwXor` | `body` | `body<-` |
| `bquote` | `browser` | `browserCondition` | `browserSetDebug` |
| `browserText` | `builtins` | `by` | `by.data.frame` |
| `by.default` | `bzfile` | `c.Date` | `c.difftime` |
| `c.factor` | `c.noquote` | `c.numeric_version` | `c.POSIXct` |
| `c.POSIXlt` | `c.warnings` | `call` | `callCC` |
| `capabilities` | `casefold` | `cat` | `cbind.data.frame` |
| `char.expand` | `character` | `charmatch` | `charToRaw` |
| `chkDots` | `chol` | `chol.default` | `chol2inv` |
| `chooseOpsMethod` | `chooseOpsMethod.default` | `class<-` | `clearPushBack` |
| `close` | `close.connection` | `close.srcfile` | `close.srcfilealias` |
| `closeAllConnections` | `colnames` | `colnames<-` | `commandArgs` |
| `comment` | `comment<-` | `complex` | `computeRestarts` |
| `conditionCall` | `conditionCall.condition` | `conditionMessage` | `conditionMessage.condition` |
| `conflictRules` | `conflicts` | `Conj` | `contributors` |
| `cospi` | `Cstack_info` | `curlGetHeaders` | `cut` |
| `cut.Date` | `cut.default` | `cut.POSIXt` | `data.class` |
| `data.matrix` | `date` | `debug` | `debuggingState` |
| `debugonce` | `declare` | `default.stringsAsFactors` | `delayedAssign` |
| `deparse` | `deparse1` | `det` | `detach` |
| `determinant` | `determinant.matrix` | `dget` | `diag<-` |
| `diff.Date` | `diff.default` | `diff.difftime` | `diff.POSIXt` |
| `difftime` | `dim.data.frame` | `dim<-` | `dimnames` |
| `dimnames.data.frame` | `dimnames<-` | `dimnames<-.data.frame` | `dir` |
| `dir.create` | `dir.exists` | `dirname` | `do.call` |
| `dontCheck` | `double` | `dput` | `dQuote` |
| `drop` | `droplevels` | `droplevels.data.frame` | `droplevels.factor` |
| `dump` | `duplicated.array` | `duplicated.data.frame` | `duplicated.default` |
| `duplicated.matrix` | `duplicated.numeric_version` | `duplicated.POSIXlt` | `duplicated.warnings` |
| `dyn.load` | `dyn.unload` | `dynGet` | `eapply` |
| `eigen` | `emptyenv` | `enc2native` | `enc2utf8` |
| `encodeString` | `Encoding` | `Encoding<-` | `enquote` |
| `env.profile` | `environment` | `environment<-` | `environmentIsLocked` |
| `environmentName` | `errorCondition` | `eval` | `eval.parent` |
| `evalq` | `Exec` | `exists` | `expand.grid` |
| `expression` | `extSoftVersion` | `fifo` | `file` |
| `file.access` | `file.append` | `file.choose` | `file.copy` |
| `file.create` | `file.exists` | `file.info` | `file.link` |
| `file.mode` | `file.mtime` | `file.path` | `file.remove` |
| `file.rename` | `file.show` | `file.size` | `file.symlink` |
| `Filter` | `Find` | `find.package` | `findInterval` |
| `findPackageEnv` | `findRestart` | `flush` | `flush.connection` |
| `force` | `forceAndCall` | `formals` | `formals<-` |
| `format` | `format.AsIs` | `format.data.frame` | `format.Date` |
| `format.default` | `format.difftime` | `format.factor` | `format.hexmode` |
| `format.info` | `format.libraryIQR` | `format.numeric_version` | `format.octmode` |
| `format.packageInfo` | `format.POSIXct` | `format.POSIXlt` | `format.pval` |
| `format.summaryDefault` | `formatC` | `formatDL` | `forwardsolve` |
| `gc` | `gc.time` | `gcinfo` | `gctorture` |
| `gctorture2` | `get` | `get0` | `getAllConnections` |
| `getCallingDLL` | `getCallingDLLe` | `getConnection` | `getDLLRegisteredRoutines` |
| `getDLLRegisteredRoutines.character` | `getDLLRegisteredRoutines.DLLInfo` | `getElement` | `geterrmessage` |
| `getExportedValue` | `getHook` | `getLoadedDLLs` | `getNamespace` |
| `getNamespaceExports` | `getNamespaceImports` | `getNamespaceInfo` | `getNamespaceName` |
| `getNamespaceUsers` | `getNamespaceVersion` | `getNativeSymbolInfo` | `getOption` |
| `getRversion` | `getSrcLines` | `getTaskCallbackNames` | `gettext` |
| `gettextf` | `getwd` | `gl` | `globalCallingHandlers` |
| `globalenv` | `gregexec` | `gregexpr` | `grepRaw` |
| `grepv` | `grouping` | `gzcon` | `gzfile` |
| `I` | `iconv` | `iconvlist` | `icuGetCollate` |
| `icuSetCollate` | `identity` | `Im` | `importIntoEnv` |
| `infoRDS` | `inherits` | `integer` | `interaction` |
| `interactive` | `intToBits` | `intToUtf8` | `inverse.rle` |
| `invisible` | `invokeRestart` | `invokeRestartInteractively` | `is.array` |
| `is.call` | `is.complex` | `is.element` | `is.environment` |
| `is.expression` | `is.finite.POSIXlt` | `is.infinite.POSIXlt` | `is.language` |
| `is.loaded` | `is.na.data.frame` | `is.na.numeric_version` | `is.na.POSIXlt` |
| `is.na<-` | `is.na<-.default` | `is.na<-.factor` | `is.na<-.numeric_version` |
| `is.name` | `is.nan.POSIXlt` | `is.numeric_version` | `is.numeric.Date` |
| `is.numeric.difftime` | `is.numeric.POSIXt` | `is.object` | `is.package_version` |
| `is.pairlist` | `is.primitive` | `is.qr` | `is.R` |
| `is.raw` | `is.recursive` | `is.single` | `is.symbol` |
| `is.table` | `isa` | `isatty` | `isBaseNamespace` |
| `isdebugged` | `isIncomplete` | `isNamespace` | `isNamespaceLoaded` |
| `ISOdate` | `ISOdatetime` | `isOpen` | `isRestart` |
| `isS4` | `isSeekable` | `isSymmetric` | `isSymmetric.matrix` |
| `jitter` | `julian` | `julian.Date` | `julian.POSIXt` |
| `kappa` | `kappa.default` | `kappa.lm` | `kappa.qr` |
| `kronecker` | `l10n_info` | `La_library` | `La_version` |
| `La.svd` | `labels` | `labels.default` | `lazyLoad` |
| `lazyLoadDBexec` | `lazyLoadDBfetch` | `length.POSIXlt` | `length<-` |
| `length<-.Date` | `length<-.difftime` | `length<-.factor` | `length<-.POSIXct` |
| `length<-.POSIXlt` | `letters` | `LETTERS` | `levels.default` |
| `levels<-` | `levels<-.factor` | `libcurlVersion` | `library` |
| `library.dynam` | `library.dynam.unload` | `licence` | `license` |
| `list.dirs` | `list.files` | `list2DF` | `list2env` |
| `load` | `loadedNamespaces` | `loadingNamespaceInfo` | `loadNamespace` |
| `local` | `lockBinding` | `lockEnvironment` | `logical` |
| `ls` | `make.names` | `make.unique` | `makeActiveBinding` |
| `Map` | `mapply` | `margin.table` | `marginSums` |
| `mat.or.vec` | `match.arg` | `match.call` | `match.fun` |
| `Math.data.frame` | `Math.Date` | `Math.difftime` | `Math.factor` |
| `Math.POSIXt` | `max.col` | `mean.Date` | `mean.default` |
| `mean.difftime` | `mean.POSIXct` | `mean.POSIXlt` | `mem.maxNSize` |
| `mem.maxVSize` | `memCompress` | `memDecompress` | `memory.profile` |
| `merge` | `merge.data.frame` | `merge.default` | `message` |
| `messageCondition` | `mget` | `Mod` | `mode<-` |
| `month.abb` | `month.name` | `months` | `months.Date` |
| `months.POSIXt` | `mostattributes<-` | `mtfrm` | `mtfrm.Date` |
| `mtfrm.default` | `mtfrm.POSIXct` | `mtfrm.POSIXlt` | `nameOfClass` |
| `nameOfClass.default` | `names.POSIXlt` | `names<-.POSIXlt` | `namespaceExport` |
| `namespaceImport` | `namespaceImportClasses` | `namespaceImportFrom` | `namespaceImportMethods` |
| `nargs` | `NCOL` | `Negate` | `new.env` |
| `NextMethod` | `ngettext` | `noquote` | `norm` |
| `normalizePath` | `NROW` | `nullfile` | `numeric` |
| `numeric_version` | `numToBits` | `numToInts` | `objects` |
| `oldClass` | `oldClass<-` | `OlsonNames` | `on.exit` |
| `open` | `open.connection` | `open.srcfile` | `open.srcfilealias` |
| `open.srcfilecopy` | `Ops.data.frame` | `Ops.Date` | `Ops.difftime` |
| `Ops.factor` | `Ops.numeric_version` | `Ops.ordered` | `Ops.POSIXt` |
| `options` | `outer` | `package_version` | `packageEvent` |
| `packageHasNamespace` | `packageNotFoundError` | `packageStartupMessage` | `packBits` |
| `pairlist` | `parent.env` | `parent.env<-` | `parent.frame` |
| `parse` | `parseNamespaceFile` | `path.expand` | `path.package` |
| `pcre_config` | `pi` | `pipe` | `plot` |
| `pmatch` | `pmax.int` | `pmin.int` | `polyroot` |
| `pos.to.env` | `Position` | `pretty` | `pretty.default` |
| `prettyNum` | `print.AsIs` | `print.by` | `print.condition` |
| `print.connection` | `print.data.frame` | `print.Date` | `print.default` |
| `print.difftime` | `print.Dlist` | `print.DLLInfo` | `print.DLLInfoList` |
| `print.DLLRegisteredRoutines` | `print.eigen` | `print.factor` | `print.function` |
| `print.hexmode` | `print.libraryIQR` | `print.listof` | `print.NativeRoutineList` |
| `print.noquote` | `print.numeric_version` | `print.octmode` | `print.packageInfo` |
| `print.POSIXct` | `print.POSIXlt` | `print.proc_time` | `print.restart` |
| `print.rle` | `print.simple.list` | `print.srcfile` | `print.srcref` |
| `print.summary.table` | `print.summary.warnings` | `print.summaryDefault` | `print.table` |
| `print.warnings` | `prmatrix` | `proc.time` | `prop.table` |
| `proportions` | `provideDimnames` | `pushBack` | `pushBackLength` |
| `q` | `qr` | `qr.coef` | `qr.default` |
| `qr.fitted` | `qr.Q` | `qr.qty` | `qr.qy` |
| `qr.R` | `qr.resid` | `qr.solve` | `qr.X` |
| `quarters` | `quarters.Date` | `quarters.POSIXt` | `quit` |
| `quote` | `R_compiled_by` | `R_system_version` | `R.home` |
| `R.version` | `R.Version` | `R.version.string` | `range.Date` |
| `range.default` | `range.POSIXct` | `rapply` | `raw` |
| `rawConnection` | `rawConnectionValue` | `rawShift` | `rawToBits` |
| `rawToChar` | `rbind.data.frame` | `rcond` | `Re` |
| `read.dcf` | `readBin` | `readChar` | `readline` |
| `readLines` | `readRDS` | `readRenviron` | `Recall` |
| `Reduce` | `reg.finalizer` | `regexec` | `regexpr` |
| `registerS3method` | `registerS3methods` | `regmatches` | `regmatches<-` |
| `remove` | `removeTaskCallback` | `rep.Date` | `rep.difftime` |
| `rep.factor` | `rep.numeric_version` | `rep.POSIXct` | `rep.POSIXlt` |
| `replicate` | `require` | `requireNamespace` | `restartDescription` |
| `restartFormals` | `retracemem` | `returnValue` | `rev.default` |
| `rle` | `rm` | `RNGkind` | `RNGversion` |
| `round.Date` | `round.POSIXt` | `row.names` | `row.names.data.frame` |
| `row.names.default` | `row.names<-` | `row.names<-.data.frame` | `row.names<-.default` |
| `rownames` | `rownames<-` | `rowsum` | `rowsum.data.frame` |
| `rowsum.default` | `save` | `save.image` | `saveRDS` |
| `scale` | `scale.default` | `scan` | `search` |
| `searchpaths` | `seek` | `seek.connection` | `seq.Date` |
| `seq.default` | `seq.POSIXt` | `sequence` | `sequence.default` |
| `serialize` | `serverSocket` | `setHook` | `setNamespaceInfo` |
| `setSessionTimeLimit` | `setTimeLimit` | `setwd` | `showConnections` |
| `shQuote` | `signalCondition` | `simpleCondition` | `simpleError` |
| `simpleMessage` | `simpleWarning` | `simplify2array` | `single` |
| `sink` | `sink.number` | `sinpi` | `slice.index` |
| `socketAccept` | `socketConnection` | `socketSelect` | `socketTimeout` |
| `solve` | `solve.default` | `solve.qr` | `sort_by` |
| `sort_by.data.frame` | `sort_by.default` | `sort.default` | `sort.int` |
| `sort.list` | `sort.POSIXlt` | `source` | `split.data.frame` |
| `split.Date` | `split.default` | `split.POSIXct` | `split<-` |
| `split<-.data.frame` | `split<-.default` | `sprintf` | `sQuote` |
| `srcfile` | `srcfilealias` | `srcfilecopy` | `srcref` |
| `standardGeneric` | `stderr` | `stdin` | `stdout` |
| `stop` | `stopifnot` | `storage.mode` | `storage.mode<-` |
| `str2expression` | `str2lang` | `strftime` | `strptime` |
| `strtoi` | `structure` | `strwrap` | `subset.data.frame` |
| `subset.default` | `subset.matrix` | `substitute` | `substr<-` |
| `substring<-` | `summary` | `summary.connection` | `summary.data.frame` |
| `Summary.data.frame` | `summary.Date` | `Summary.Date` | `summary.default` |
| `summary.difftime` | `Summary.difftime` | `summary.factor` | `Summary.factor` |
| `summary.matrix` | `Summary.numeric_version` | `Summary.ordered` | `summary.POSIXct` |
| `Summary.POSIXct` | `summary.POSIXlt` | `Summary.POSIXlt` | `summary.proc_time` |
| `summary.srcfile` | `summary.srcref` | `summary.table` | `summary.warnings` |
| `suppressMessages` | `suppressPackageStartupMessages` | `suppressWarnings` | `suspendInterrupts` |
| `svd` | `sweep` | `switch` | `sys.call` |
| `sys.calls` | `Sys.chmod` | `Sys.Date` | `sys.frame` |
| `sys.frames` | `sys.function` | `Sys.getenv` | `Sys.getlocale` |
| `Sys.getpid` | `Sys.glob` | `Sys.info` | `sys.load.image` |
| `Sys.localeconv` | `sys.nframe` | `sys.on.exit` | `sys.parent` |
| `sys.parents` | `Sys.readlink` | `sys.save.image` | `Sys.setenv` |
| `Sys.setFileTime` | `Sys.setLanguage` | `Sys.setlocale` | `Sys.sleep` |
| `sys.source` | `sys.status` | `Sys.time` | `Sys.timezone` |
| `Sys.umask` | `Sys.unsetenv` | `Sys.which` | `system` |
| `system.file` | `system.time` | `system2` | `t.data.frame` |
| `t.default` | `Tailcall` | `tanpi` | `tapply` |
| `taskCallbackManager` | `tempdir` | `tempfile` | `textConnection` |
| `textConnectionValue` | `topenv` | `toString` | `toString.default` |
| `trace` | `traceback` | `tracemem` | `tracingState` |
| `transform.data.frame` | `transform.default` | `trunc.Date` | `trunc.POSIXt` |
| `truncate` | `truncate.connection` | `try` | `tryCatch` |
| `tryInvokeRestart` | `unCfillPOSIXlt` | `unclass` | `undebug` |
| `unique.array` | `unique.data.frame` | `unique.default` | `unique.matrix` |
| `unique.numeric_version` | `unique.POSIXlt` | `unique.warnings` | `units` |
| `units.difftime` | `units<-` | `units<-.difftime` | `unix.time` |
| `unlink` | `unloadNamespace` | `unlockBinding` | `unname` |
| `unserialize` | `unsplit` | `untrace` | `untracemem` |
| `unz` | `url` | `use` | `UseMethod` |
| `utf8ToInt` | `validEnc` | `validUTF8` | `vapply` |
| `vector` | `Vectorize` | `version` | `warning` |
| `warningCondition` | `warnings` | `weekdays` | `weekdays.Date` |
| `weekdays.POSIXt` | `with.default` | `withAutoprint` | `withCallingHandlers` |
| `within` | `within.data.frame` | `within.list` | `withRestarts` |
| `withVisible` | `write` | `write.dcf` | `writeBin` |
| `writeChar` | `writeLines` | `xpdrows.data.frame` | `xtfrm` |
| `xtfrm.AsIs` | `xtfrm.data.frame` | `xtfrm.Date` | `xtfrm.default` |
| `xtfrm.difftime` | `xtfrm.factor` | `xtfrm.numeric_version` | `xtfrm.POSIXct` |
| `xtfrm.POSIXlt` | `xzfile` | `zapsmall` | `zstdfile` |

S4 class and generic metadata objects, missing with the S4 system (3): `.__H__.cbind`, `.__H__.rbind`, `.__S3MethodsTable__.`.

### stats

465 objects: 57 done, 24 partial, 384 missing.

| Name | Kind | Status | Notes | Evidence |
| --- | --- | --- | --- | --- |
| `complete.cases` | closure | Done | Values match; printing differs (RC8). |  |
| `dbeta` | closure | Done |  |  |
| `dbinom` | closure | Done | Values match; printing differs (RC8). |  |
| `dcauchy` | closure | Done | Values match; printing differs (RC8). |  |
| `dchisq` | closure | Done | Values match; printing differs (RC8). |  |
| `dexp` | closure | Done |  |  |
| `df` | closure | Done | Values match; printing differs (RC8). |  |
| `dgamma` | closure | Partial | Arguments not referenced: `rate`. No `rate` argument; the third positional argument is read as `scale` (RC6). | <code>dgamma(1.5, 2, 3)</code>: R <code>[1] 0.149971453266</code>, R-ust <code>[1] 0.101088443285</code> |
| `dgeom` | closure | Done | Values match; printing differs (RC8). |  |
| `dhyper` | closure | Done |  |  |
| `dlnorm` | closure | Done | Values match; printing differs (RC8). |  |
| `dlogis` | closure | Done |  |  |
| `dnbinom` | closure | Done | Values match; printing differs (RC8). |  |
| `dnorm` | closure | Done |  |  |
| `dpois` | closure | Done | Values match; printing differs (RC8). |  |
| `dsignrank` | closure | Done | Values match; printing differs (RC8). |  |
| `dt` | closure | Done | Values match; printing differs (RC8). |  |
| `dunif` | closure | Done |  |  |
| `dweibull` | closure | Done | Values match; printing differs (RC8). |  |
| `dwilcox` | closure | Done | Values match; printing differs (RC8). |  |
| `median` | closure | Done |  |  |
| `na.omit` | closure | Partial | The `na.action` attribute and `omit` class are not attached (RC3). | <code>na.omit(c(1, NA, 3))</code>: R <code>[1] 1 3 ⏎ attr(,&quot;na.action&quot;) ⏎ [1] 2 ⏎ attr(,&quot;class&quot;) ⏎ [1] …</code>, R-ust <code>[1] 1 3</code> |
| `pbeta` | closure | Done |  |  |
| `pbinom` | closure | Done |  |  |
| `pcauchy` | closure | Done |  |  |
| `pchisq` | closure | Done |  |  |
| `pexp` | closure | Done |  |  |
| `pf` | closure | Done |  |  |
| `pgamma` | closure | Partial | Arguments not referenced: `rate`. No `rate` argument; the third positional argument is read as `scale` (RC6). | <code>pgamma(1.5, 2, 3)</code>: R <code>[1] 0.93890051904</code>, R-ust <code>[1] 0.090204010431</code> |
| `pgeom` | closure | Done |  |  |
| `phyper` | closure | Done |  |  |
| `plnorm` | closure | Done |  |  |
| `plogis` | closure | Done |  |  |
| `pnbinom` | closure | Done |  |  |
| `pnorm` | closure | Done | Values match; printing differs (RC8). |  |
| `ppois` | closure | Done |  |  |
| `psignrank` | closure | Done |  |  |
| `pt` | closure | Done | Values match; printing differs (RC8). |  |
| `punif` | closure | Done |  |  |
| `pweibull` | closure | Done |  |  |
| `pwilcox` | closure | Done | Values match; printing differs (RC8). |  |
| `qbeta` | closure | Done |  |  |
| `qbinom` | closure | Done |  |  |
| `qcauchy` | closure | Done |  |  |
| `qchisq` | closure | Done | Values match; printing differs (RC8). |  |
| `qexp` | closure | Done |  |  |
| `qf` | closure | Done | Values match; printing differs (RC8). |  |
| `qgamma` | closure | Partial | Arguments not referenced: `rate`. No `rate` argument; the third positional argument is read as `scale` (RC6). | <code>qgamma(0.5, 2, 3)</code>: R <code>[1] 0.559448996672</code>, R-ust <code>[1] 5.03504097005</code> |
| `qgeom` | closure | Done |  |  |
| `qhyper` | closure | Done |  |  |
| `qlnorm` | closure | Done |  |  |
| `qlogis` | closure | Done |  |  |
| `qnbinom` | closure | Done |  |  |
| `qnorm` | closure | Done |  |  |
| `qpois` | closure | Done |  |  |
| `qsignrank` | closure | Done |  |  |
| `qt` | closure | Done | Values match; printing differs (RC8). |  |
| `qunif` | closure | Done |  |  |
| `qweibull` | closure | Done |  |  |
| `qwilcox` | closure | Done |  |  |
| `rbeta` | closure | Partial | Arguments not referenced: `ncp`. Draws differ from R (RC5). | <code>set.seed(42); rbeta(3, 2, 5)</code>: R <code>[1] 0.184935757350 0.364760396264 0.430807010969</code>, R-ust <code>[1] 0.331160474066 0.214828681156 0.293251694372</code> |
| `rbinom` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rbinom(3, 10, 0.3)</code>: R <code>[1] 5 5 2</code>, R-ust <code>[1] 5 5 4</code> |
| `rcauchy` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rcauchy(3)</code>: R <code>[1] -0.274224087127 -0.200299391087  1.257402750519</code>, R-ust <code>[1] -1.448804996787 1.53217687328 0.41797738343</code> |
| `rchisq` | closure | Partial | Arguments not referenced: `ncp`. Draws differ from R (RC5). | <code>set.seed(42); rchisq(3, 3)</code>: R <code>[1] 5.68168042619 1.03004566960 2.09691594616</code>, R-ust <code>[1] 3.595568495405 2.522449268677 1.039877503811</code> |
| `rexp` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rexp(3, 2)</code>: R <code>[1] 0.0991684058919 0.3304476260673 0.1417455188930</code>, R-ust <code>[1] 0.192302054887 0.478449978526 0.697232168243</code> |
| `rf` | closure | Partial | Arguments not referenced: `ncp`. Draws differ from R (RC5). | <code>set.seed(42); rf(3, 3, 10)</code>: R <code>[1] 2.800113717836 1.419621172791 0.930186154497</code>, R-ust <code>[1] 1.234743667201 0.550700356226 0.820663037031</code> |
| `rgamma` | closure | Partial | Draws differ from R (RC5). No `rate` argument; the third positional argument is read as `scale` (RC6). | <code>set.seed(42); rgamma(3, 2, 3)</code>: R <code>[1] 1.216318697662 0.296036605631 0.519739940646</code>, R-ust <code>[1] 7.209038230041 5.337612466361 2.583239454455</code> |
| `rgeom` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rgeom(3, 0.3)</code>: R <code>[1] 1 1 0</code>, R-ust <code>[1] 0 2 3</code> |
| `rhyper` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rhyper(3, 5, 10, 4)</code>: R <code>[1] 2 3 1</code>, R-ust <code>[1] 2 3 2</code> |
| `rlnorm` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rlnorm(3)</code>: R <code>[1] 3.939124329241 0.568531719999 1.437820479838</code>, R-ust <code>[1] 1.652660112981 0.31809588303 0.529624487344</code> |
| `rlogis` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rlogis(3)</code>: R <code>[1]  2.373781570221  2.700826789878 -0.914207940658</code>, R-ust <code>[1] 0.810903751792 -0.772501666771 -1.936605546291</code> |
| `rnbinom` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rnbinom(3, 5, 0.5)</code>: R <code>[1] 6 7 7</code>, R-ust <code>[1] 4 4 2</code> |
| `rnorm` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rnorm(3)</code>: R <code>[1]  1.370958447147 -0.564698171396  0.363128411337</code>, R-ust <code>[1] 0.502386179415 -1.145402422688 -0.635587037996</code> |
| `rpois` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rpois(3, 3)</code>: R <code>[1] 5 6 2</code>, R-ust <code>[1] 2 2 2</code> |
| `rsignrank` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rsignrank(3, 10)</code>: R <code>[1] 44 28 32</code>, R-ust <code>[1] 50 35 20</code> |
| `rt` | closure | Partial | Arguments not referenced: `ncp`. Draws differ from R (RC5). | <code>set.seed(42); rt(3, 5)</code>: R <code>[1]  1.9151370856547  0.0878422087160 -0.0773269863273</code>, R-ust <code>[1] 0.846682815381 -1.353938439043 1.106618857088</code> |
| `runif` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); runif(3, 0, 2)</code>: R <code>[1] 1.829612086993 1.874150826596 0.572279069573</code>, R-ust <code>[1] 1.384604109774 0.631876388246 0.252042493842</code> |
| `rweibull` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rweibull(3, 2, 1)</code>: R <code>[1] 0.298401089973 0.254934337049 1.118604355110</code>, R-ust <code>[1] 0.606409863953 1.073409834473 1.439202820971</code> |
| `rwilcox` | closure | Partial | Draws differ from R (RC5). | <code>set.seed(42); rwilcox(3, 4, 6)</code>: R <code>[1]  9 10  2</code>, R-ust <code>[1] 9 12 17</code> |
| `sd` | closure | Done |  |  |
| `var` | closure | Partial | Arguments not referenced: `y`, `use`. |  |

Missing (384):

|   |   |   |   |
| --- | --- | --- | --- |
| `.checkMFClasses` | `.getXlevels` | `.lm.fit` | `.MFclass` |
| `.nknots.smspl` | `.preformat.ts` | `.vcov.aliased` | `acf` |
| `acf2AR` | `add.scope` | `add1` | `addmargins` |
| `aggregate` | `aggregate.data.frame` | `aggregate.ts` | `AIC` |
| `alias` | `anova` | `ansari.test` | `aov` |
| `approx` | `approxfun` | `ar` | `ar.burg` |
| `ar.mle` | `ar.ols` | `ar.yw` | `arima` |
| `arima.sim` | `arima0` | `arima0.diag` | `ARMAacf` |
| `ARMAtoMA` | `as.dendrogram` | `as.dist` | `as.formula` |
| `as.hclust` | `as.stepfun` | `as.ts` | `asOneSidedFormula` |
| `ave` | `bandwidth.kernel` | `bartlett.test` | `BIC` |
| `binom.test` | `binomial` | `biplot` | `Box.test` |
| `bw.bcv` | `bw.nrd` | `bw.nrd0` | `bw.SJ` |
| `bw.ucv` | `C` | `cancor` | `case.names` |
| `ccf` | `chisq.test` | `cmdscale` | `coef` |
| `coefficients` | `confint` | `confint.default` | `confint.lm` |
| `constrOptim` | `contr.helmert` | `contr.poly` | `contr.SAS` |
| `contr.sum` | `contr.treatment` | `contrasts` | `contrasts<-` |
| `convolve` | `cooks.distance` | `cophenetic` | `cor` |
| `cor.test` | `cov` | `cov.wt` | `cov2cor` |
| `covratio` | `cpgram` | `cutree` | `cycle` |
| `D` | `decompose` | `delete.response` | `deltat` |
| `dendrapply` | `density` | `density.default` | `deriv` |
| `deriv3` | `deviance` | `df.kernel` | `df.residual` |
| `DF2formula` | `dfbeta` | `dfbetas` | `dffits` |
| `diffinv` | `dist` | `dmultinom` | `drop.scope` |
| `drop.terms` | `drop1` | `dummy.coef` | `dummy.coef.lm` |
| `ecdf` | `eff.aovlist` | `effects` | `embed` |
| `end` | `estVar` | `expand.model.frame` | `extractAIC` |
| `factanal` | `factor.scope` | `family` | `fft` |
| `filter` | `fisher.test` | `fitted` | `fitted.values` |
| `fivenum` | `fligner.test` | `formula` | `free1way` |
| `frequency` | `friedman.test` | `ftable` | `Gamma` |
| `gaussian` | `get_all_vars` | `getCall` | `getInitial` |
| `glm` | `glm.control` | `glm.fit` | `hasTsp` |
| `hat` | `hatvalues` | `hclust` | `heatmap` |
| `HoltWinters` | `influence` | `influence.measures` | `integrate` |
| `interaction.plot` | `inverse.gaussian` | `IQR` | `is.empty.model` |
| `is.leaf` | `is.mts` | `is.stepfun` | `is.ts` |
| `is.tskernel` | `isoreg` | `KalmanForecast` | `KalmanLike` |
| `KalmanRun` | `KalmanSmooth` | `kernapply` | `kernel` |
| `kmeans` | `knots` | `kruskal.test` | `ks.test` |
| `ksmooth` | `lag` | `lag.plot` | `line` |
| `lm` | `lm.fit` | `lm.influence` | `lm.wfit` |
| `loadings` | `loess` | `loess.control` | `loess.smooth` |
| `logLik` | `loglin` | `lowess` | `ls.diag` |
| `ls.print` | `lsfit` | `mad` | `mahalanobis` |
| `make.link` | `makeARIMA` | `makepredictcall` | `manova` |
| `mantelhaen.test` | `mauchly.test` | `mcnemar.test` | `median.default` |
| `medpolish` | `model.extract` | `model.frame` | `model.frame.default` |
| `model.matrix` | `model.matrix.default` | `model.matrix.lm` | `model.offset` |
| `model.response` | `model.tables` | `model.weights` | `monthplot` |
| `mood.test` | `mvfft` | `na.action` | `na.contiguous` |
| `na.exclude` | `na.fail` | `na.pass` | `napredict` |
| `naprint` | `naresid` | `nextn` | `nlm` |
| `nlminb` | `nls` | `nls.control` | `NLSstAsymptotic` |
| `NLSstClosestX` | `NLSstLfAsymptote` | `NLSstRtAsymptote` | `nobs` |
| `numericDeriv` | `offset` | `oneway.test` | `optim` |
| `optimHess` | `optimise` | `optimize` | `order.dendrogram` |
| `p.adjust` | `p.adjust.methods` | `pacf` | `Pair` |
| `pairwise.prop.test` | `pairwise.t.test` | `pairwise.table` | `pairwise.wilcox.test` |
| `pbirthday` | `plclust` | `plot.ecdf` | `plot.spec.coherency` |
| `plot.spec.phase` | `plot.stepfun` | `plot.ts` | `poisson` |
| `poisson.test` | `poly` | `polym` | `power` |
| `power.anova.test` | `power.free1way.test` | `power.prop.test` | `power.t.test` |
| `PP.test` | `ppoints` | `ppplot` | `ppr` |
| `prcomp` | `predict` | `predict.glm` | `predict.lm` |
| `preplot` | `princomp` | `printCoefmat` | `profile` |
| `proj` | `promax` | `prop.test` | `prop.trend.test` |
| `psmirnov` | `ptukey` | `qbirthday` | `qqline` |
| `qqnorm` | `qqplot` | `qr.influence` | `qsmirnov` |
| `qtukey` | `quade.test` | `quantile` | `quasi` |
| `quasibinomial` | `quasipoisson` | `r2dtable` | `read.ftable` |
| `rect.hclust` | `reformulate` | `relevel` | `reorder` |
| `replications` | `reshape` | `resid` | `residuals` |
| `residuals.glm` | `residuals.lm` | `rfree1way` | `rmultinom` |
| `rsmirnov` | `rstandard` | `rstudent` | `runmed` |
| `rWishart` | `scatter.smooth` | `screeplot` | `se.contrast` |
| `selfStart` | `setNames` | `shapiro.test` | `sigma` |
| `simulate` | `smooth` | `smooth.spline` | `smoothEnds` |
| `sortedXyData` | `spec.ar` | `spec.pgram` | `spec.taper` |
| `spectrum` | `spline` | `splinefun` | `splinefunH` |
| `SSasymp` | `SSasympOff` | `SSasympOrig` | `SSbiexp` |
| `SSD` | `SSfol` | `SSfpl` | `SSgompertz` |
| `SSlogis` | `SSmicmen` | `SSweibull` | `start` |
| `stat.anova` | `step` | `stepfun` | `stl` |
| `StructTS` | `summary.aov` | `summary.glm` | `summary.lm` |
| `summary.manova` | `summary.stepfun` | `supsmu` | `symnum` |
| `t.test` | `termplot` | `terms` | `terms.formula` |
| `time` | `toeplitz` | `toeplitz2` | `ts` |
| `ts.intersect` | `ts.plot` | `ts.union` | `tsdiag` |
| `tsp` | `tsp<-` | `tsSmooth` | `TukeyHSD` |
| `uniroot` | `update` | `update.default` | `update.formula` |
| `var.test` | `variable.names` | `varimax` | `vcov` |
| `weighted.mean` | `weighted.residuals` | `weights` | `wilcox.test` |
| `window` | `window<-` | `write.ftable` | `xtabs` |

### utils

237 objects: 1 done, 1 partial, 235 missing.

| Name | Kind | Status | Notes | Evidence |
| --- | --- | --- | --- | --- |
| `head` | closure | Done |  |  |
| `tail` | closure | Partial | Arguments not referenced: `keepnums`, `addrownums`. Values match; printing differs (RC8). |  |

Missing (235):

|   |   |   |   |
| --- | --- | --- | --- |
| `?` | `.AtNames` | `.checkHT` | `.DollarNames` |
| `.romans` | `.RtangleCodeLabel` | `.S3methods` | `adist` |
| `alarm` | `apropos` | `aregexec` | `argsAnywhere` |
| `as.person` | `as.personList` | `as.relistable` | `as.roman` |
| `asDateBuilt` | `askYesNo` | `aspell` | `aspell_package_C_files` |
| `aspell_package_R_files` | `aspell_package_Rd_files` | `aspell_package_vignettes` | `aspell_write_personal_dictionary_file` |
| `assignInMyNamespace` | `assignInNamespace` | `available.packages` | `bibentry` |
| `bitstring` | `browseEnv` | `browseURL` | `browseVignettes` |
| `bug.report` | `capture.output` | `changedFiles` | `charClass` |
| `checkCRAN` | `chooseBioCmirror` | `chooseCRANmirror` | `citation` |
| `cite` | `citeNatbib` | `citEntry` | `citFooter` |
| `citHeader` | `close.socket` | `clrhash` | `combn` |
| `compareVersion` | `contrib.url` | `count.fields` | `create.post` |
| `data` | `data.entry` | `dataentry` | `de` |
| `de.ncols` | `de.restore` | `de.setup` | `debugcall` |
| `debugger` | `demo` | `download.file` | `download.packages` |
| `dump.frames` | `edit` | `emacs` | `example` |
| `file_test` | `file.edit` | `fileSnapshot` | `find` |
| `findCRANmirror` | `findLineNum` | `findMatches` | `fix` |
| `fixInNamespace` | `flush.console` | `formatOL` | `formatUL` |
| `getAnywhere` | `getCRANmirrors` | `getFromNamespace` | `gethash` |
| `getParseData` | `getParseText` | `getS3method` | `getSrcDirectory` |
| `getSrcFilename` | `getSrcLocation` | `getSrcref` | `getTxtProgressBar` |
| `glob2rx` | `globalVariables` | `hashtab` | `hasName` |
| `head.matrix` | `help` | `help.request` | `help.search` |
| `help.start` | `history` | `hsearch_db` | `hsearch_db_concepts` |
| `hsearch_db_keywords` | `install.packages` | `installed.packages` | `is.hashtab` |
| `is.relistable` | `isS3method` | `isS3stdGeneric` | `limitedLabels` |
| `loadhistory` | `localeToCharset` | `ls.str` | `lsf.str` |
| `maintainer` | `make.packages.html` | `make.socket` | `makeRweaveLatexCodeRunner` |
| `maphash` | `memory.limit` | `memory.size` | `menu` |
| `methods` | `mirror2html` | `modifyList` | `new.packages` |
| `news` | `nsl` | `numhash` | `object.size` |
| `old.packages` | `osVersion` | `package.skeleton` | `packageDate` |
| `packageDescription` | `packageName` | `packageStatus` | `packageVersion` |
| `page` | `person` | `personList` | `pico` |
| `process.events` | `prompt` | `promptData` | `promptImport` |
| `promptPackage` | `rc.getOption` | `rc.options` | `rc.settings` |
| `rc.status` | `read.csv` | `read.csv2` | `read.delim` |
| `read.delim2` | `read.DIF` | `read.fortran` | `read.fwf` |
| `read.socket` | `read.table` | `readCitationFile` | `recover` |
| `relist` | `remhash` | `remove.packages` | `removeSource` |
| `Rprof` | `Rprofmem` | `RShowDoc` | `RSiteSearch` |
| `rtags` | `Rtangle` | `RtangleFinish` | `RtangleOptions` |
| `RtangleRuncode` | `RtangleSetup` | `RtangleWritedoc` | `RweaveChunkPrefix` |
| `RweaveEvalWithOpt` | `RweaveLatex` | `RweaveLatexFinish` | `RweaveLatexOptions` |
| `RweaveLatexSetup` | `RweaveLatexWritedoc` | `RweaveTryStop` | `savehistory` |
| `select.list` | `sessionInfo` | `setBreakpoint` | `sethash` |
| `setRepositories` | `setTxtProgressBar` | `stack` | `Stangle` |
| `str` | `strcapture` | `strOptions` | `summaryRprof` |
| `suppressForeignCheck` | `Sweave` | `SweaveGetSourceName` | `SweaveHooks` |
| `SweaveSyntaxLatex` | `SweaveSyntaxNoweb` | `SweaveSyntConv` | `tail.matrix` |
| `tar` | `timestamp` | `toBibtex` | `toLatex` |
| `txtProgressBar` | `type.convert` | `typhash` | `undebugcall` |
| `unstack` | `untar` | `unzip` | `update.packages` |
| `upgrade` | `url.show` | `URLdecode` | `URLencode` |
| `vi` | `View` | `vignette` | `warnErrList` |
| `write.csv` | `write.csv2` | `write.socket` | `write.table` |
| `xedit` | `xemacs` | `zip` |  |

### methods

365 objects: 0 done, 0 partial, 365 missing.

Missing (212):

|   |   |   |   |
| --- | --- | --- | --- |
| `.classEnv` | `.doTracePrint` | `.EmptyPrimitiveSkeletons` | `.hasSlot` |
| `.OldClassesList` | `.S4methods` | `.selectSuperClasses` | `.ShortPrimitiveSkeletons` |
| `.slotNames` | `.untracedFunction` | `.valueClassTest` | `addNextMethod` |
| `allNames` | `Arith` | `as` | `as<-` |
| `asMethodDefinition` | `assignClassDef` | `assignMethodsMetaData` | `balanceMethodsList` |
| `body<-` | `cacheGenericsMetaData` | `cacheMetaData` | `cacheMethod` |
| `callGeneric` | `callNextMethod` | `canCoerce` | `cbind2` |
| `checkAtAssignment` | `checkSlotAssignment` | `classesToAM` | `classLabel` |
| `classMetaName` | `className` | `coerce` | `coerce<-` |
| `Compare` | `completeClassDefinition` | `completeExtends` | `completeSubclasses` |
| `Complex` | `conformMethod` | `defaultDumpName` | `defaultPrototype` |
| `doPrimitiveMethod` | `dumpMethod` | `dumpMethods` | `el` |
| `el<-` | `elNamed` | `elNamed<-` | `empty.dump` |
| `emptyMethodsList` | `evalOnLoad` | `evalqOnLoad` | `evalSource` |
| `existsFunction` | `existsMethod` | `extends` | `externalRefMethod` |
| `finalDefaultMethod` | `findClass` | `findFunction` | `findMethod` |
| `findMethods` | `findMethodSignatures` | `findUnique` | `fixPre1.8` |
| `formalArgs` | `functionBody` | `functionBody<-` | `generic.skeleton` |
| `getAllSuperClasses` | `getClass` | `getClassDef` | `getClasses` |
| `getDataPart` | `getFunction` | `getGeneric` | `getGenerics` |
| `getGroup` | `getGroupMembers` | `getLoadActions` | `getMethod` |
| `getMethods` | `getMethodsForDispatch` | `getMethodsMetaData` | `getPackageName` |
| `getRefClass` | `getSlots` | `getValidity` | `hasArg` |
| `hasLoadAction` | `hasMethod` | `hasMethods` | `implicitGeneric` |
| `inheritedSlotNames` | `initFieldArgs` | `initialize` | `initRefFields` |
| `insertClassMethods` | `insertMethod` | `insertSource` | `is` |
| `isClass` | `isClassDef` | `isClassUnion` | `isGeneric` |
| `isGrammarSymbol` | `isGroup` | `isRematched` | `isSealedClass` |
| `isSealedMethod` | `isVirtualClass` | `isXS3Class` | `kronecker` |
| `languageEl` | `languageEl<-` | `listFromMethods` | `loadMethod` |
| `Logic` | `makeClassRepresentation` | `makeExtends` | `makeGeneric` |
| `makeMethodsList` | `makePrototypeFromClassDef` | `makeStandardGeneric` | `matchSignature` |
| `Math` | `Math2` | `matrixOps` | `mergeMethods` |
| `metaNameUndo` | `method.skeleton` | `MethodAddCoerce` | `methodSignatureMatrix` |
| `MethodsList` | `MethodsListSelect` | `methodsPackageMetaName` | `missingArg` |
| `multipleClasses` | `new` | `newBasic` | `newClassRepresentation` |
| `newEmptyObject` | `Ops` | `packageSlot` | `packageSlot<-` |
| `possibleExtends` | `prohibitGeneric` | `promptClass` | `promptMethods` |
| `prototype` | `Quote` | `rbind2` | `reconcilePropertiesAndPrototype` |
| `registerImplicitGenerics` | `rematchDefinition` | `removeClass` | `removeGeneric` |
| `removeMethod` | `removeMethods` | `representation` | `requireMethods` |
| `resetClass` | `resetGeneric` | `S3Class` | `S3Class<-` |
| `S3Part` | `S3Part<-` | `sealClass` | `selectMethod` |
| `selectSuperClasses` | `setAs` | `setClass` | `setClassUnion` |
| `setDataPart` | `setGeneric` | `setGenericImplicit` | `setGroupGeneric` |
| `setIs` | `setLoadAction` | `setLoadActions` | `setMethod` |
| `setOldClass` | `setPackageName` | `setPrimitiveMethods` | `setRefClass` |
| `setReplaceMethod` | `setValidity` | `show` | `showClass` |
| `showDefault` | `showExtends` | `showMethods` | `signature` |
| `SignatureMethod` | `sigToEnv` | `slot` | `slot<-` |
| `slotNames` | `slotsFromS3` | `substituteDirect` | `substituteFunctionArgs` |
| `Summary` | `superClassDepth` | `testInheritedMethods` | `testVirtual` |
| `tryNew` | `unRematchDefinition` | `validObject` | `validSlotNames` |

S4 class and generic metadata objects, missing with the S4 system (153): `.__C__.environment`, `.__C__.externalptr`, `.__C__.name`, `.__C__.NULL`, `.__C__.Other`, `.__C__(`, `.__C__{`, `.__C__<-`, `.__C__activeBindingFunction`, `.__C__anova`, `.__C__anova.glm`, `.__C__anova.glm.null`, `.__C__ANY`, `.__C__aov`, `.__C__array`, `.__C__builtin`, `.__C__call`, `.__C__character`, `.__C__classGeneratorFunction`, `.__C__className`, `.__C__classPrototypeDef`, `.__C__classRepresentation`, `.__C__ClassUnionRepresentation`, `.__C__complex`, `.__C__conditionalExtension`, `.__C__data.frame`, `.__C__data.frameRowLabels`, `.__C__Date`, `.__C__defaultBindingFunction`, `.__C__density`, `.__C__derivedDefaultMethod`, `.__C__derivedDefaultMethodWithTrace`, `.__C__difftime`, `.__C__double`, `.__C__dump.frames`, `.__C__environment`, `.__C__envRefClass`, `.__C__expression`, `.__C__externalptr`, `.__C__externalRefMethod`, `.__C__factor`, `.__C__for`, `.__C__formula`, `.__C__function`, `.__C__functionWithTrace`, `.__C__genericFunction`, `.__C__genericFunctionWithTrace`, `.__C__glm`, `.__C__glm.null`, `.__C__groupGenericFunction`, `.__C__groupGenericFunctionWithTrace`, `.__C__hsearch`, `.__C__if`, `.__C__integer`, `.__C__integrate`, `.__C__internalDispatchMethod`, `.__C__language`, `.__C__libraryIQR`, `.__C__LinearMethodsList`, `.__C__list`, `.__C__listOfMethods`, `.__C__lm`, `.__C__localRefClass`, `.__C__logical`, `.__C__logLik`, `.__C__maov`, `.__C__matrix`, `.__C__MethodDefinition`, `.__C__MethodDefinitionWithTrace`, `.__C__MethodSelectionReport`, `.__C__MethodWithNext`, `.__C__MethodWithNextWithTrace`, `.__C__missing`, `.__C__mlm`, `.__C__mtable`, `.__C__mts`, `.__C__name`, `.__C__namedList`, `.__C__nonstandardGeneric`, `.__C__nonstandardGenericFunction`, `.__C__nonstandardGenericWithTrace`, `.__C__nonstandardGroupGenericFunction`, `.__C__nonStructure`, `.__C__NULL`, `.__C__numeric`, `.__C__ObjectsWithPackage`, `.__C__oldClass`, `.__C__OptionalFunction`, `.__C__optionalMethod`, `.__C__ordered`, `.__C__packageInfo`, `.__C__packageIQR`, `.__C__POSIXct`, `.__C__POSIXlt`, `.__C__POSIXt`, `.__C__PossibleMethod`, `.__C__raw`, `.__C__recordedplot`, `.__C__refClass`, `.__C__refClassRepresentation`, `.__C__refGeneratorSlot`, `.__C__refMethodDef`, `.__C__refMethodDefWithTrace`, `.__C__refObject`, `.__C__refObjectGenerator`, `.__C__repeat`, `.__C__rle`, `.__C__S3`, `.__C__S4`, `.__C__SClassExtension`, `.__C__SealedMethodDefinition`, `.__C__signature`, `.__C__socket`, `.__C__sourceEnvironment`, `.__C__special`, `.__C__standardGeneric`, `.__C__standardGenericWithTrace`, `.__C__structure`, `.__C__summary.table`, `.__C__summaryDefault`, `.__C__SuperClassMethod`, `.__C__table`, `.__C__traceable`, `.__C__ts`, `.__C__uninitializedField`, `.__C__vector`, `.__C__VIRTUAL`, `.__C__while`, `.__T__[:base`, `.__T__[[<-:base`, `.__T__[<-:base`, `.__T__$:base`, `.__T__$<-:base`, `.__T__addNextMethod:methods`, `.__T__Arith:base`, `.__T__body<-:base`, `.__T__cbind2:methods`, `.__T__coerce:methods`, `.__T__coerce<-:methods`, `.__T__Compare:methods`, `.__T__Complex:base`, `.__T__initialize:methods`, `.__T__kronecker:base`, `.__T__loadMethod:methods`, `.__T__Logic:base`, `.__T__Math:base`, `.__T__Math2:methods`, `.__T__matrixOps:base`, `.__T__Ops:base`, `.__T__rbind2:methods`, `.__T__show:methods`, `.__T__slotsFromS3:methods`, `.__T__Summary:base`.

### graphics

88 objects: 0 done, 0 partial, 88 missing.

Missing (88):

|   |   |   |   |
| --- | --- | --- | --- |
| `.filled.contour` | `abline` | `arrows` | `assocplot` |
| `axis` | `Axis` | `axis.Date` | `axis.POSIXct` |
| `axTicks` | `barplot` | `barplot.default` | `box` |
| `boxplot` | `boxplot.default` | `boxplot.matrix` | `bxp` |
| `cdplot` | `clip` | `close.screen` | `co.intervals` |
| `contour` | `contour.default` | `coplot` | `curve` |
| `dotchart` | `erase.screen` | `filled.contour` | `fourfoldplot` |
| `frame` | `grconvertX` | `grconvertY` | `grid` |
| `hist` | `hist.default` | `identify` | `image` |
| `image.default` | `layout` | `layout.show` | `lcm` |
| `legend` | `lines` | `lines.default` | `locator` |
| `matlines` | `matplot` | `matpoints` | `mosaicplot` |
| `mtext` | `pairs` | `pairs.default` | `panel.smooth` |
| `par` | `persp` | `pie` | `plot` |
| `plot.default` | `plot.design` | `plot.function` | `plot.new` |
| `plot.window` | `plot.xy` | `points` | `points.default` |
| `polygon` | `polypath` | `rasterImage` | `rect` |
| `rug` | `screen` | `segments` | `smoothScatter` |
| `spineplot` | `split.screen` | `stars` | `stem` |
| `strheight` | `stripchart` | `strwidth` | `sunflowerplot` |
| `symbols` | `text` | `text.default` | `title` |
| `xinch` | `xspline` | `xyinch` | `yinch` |

### grDevices

137 objects: 0 done, 0 partial, 137 missing.

Missing (137):

|   |   |   |   |
| --- | --- | --- | --- |
| `.axisPars` | `.clipPath` | `.defineGroup` | `.devUp` |
| `.linearGradientPattern` | `.mask` | `.opIndex` | `.radialGradientPattern` |
| `.ruleIndex` | `.setClipPath` | `.setMask` | `.setPattern` |
| `.tilingPattern` | `.useGroup` | `adjustcolor` | `as.graphicsAnnot` |
| `as.raster` | `axisTicks` | `bitmap` | `blues9` |
| `bmp` | `boxplot.stats` | `cairo_pdf` | `cairo_ps` |
| `cairoSymbolFont` | `check.options` | `chull` | `CIDFont` |
| `cm` | `cm.colors` | `col2rgb` | `colorConverter` |
| `colorRamp` | `colorRampPalette` | `colors` | `colorspaces` |
| `colours` | `contourLines` | `convertColor` | `densCols` |
| `dev.capabilities` | `dev.capture` | `dev.control` | `dev.copy` |
| `dev.copy2eps` | `dev.copy2pdf` | `dev.cur` | `dev.flush` |
| `dev.hold` | `dev.interactive` | `dev.list` | `dev.new` |
| `dev.next` | `dev.off` | `dev.prev` | `dev.print` |
| `dev.set` | `dev.size` | `dev2bitmap` | `devAskNewPage` |
| `deviceIsInteractive` | `embedFonts` | `embedGlyphs` | `extendrange` |
| `getGraphicsEvent` | `getGraphicsEventEnv` | `glyphAnchor` | `glyphFont` |
| `glyphFontList` | `glyphFontVariation` | `glyphHeight` | `glyphHeightBottom` |
| `glyphInfo` | `glyphJust` | `glyphWidth` | `glyphWidthLeft` |
| `graphics.off` | `gray` | `gray.colors` | `grey` |
| `grey.colors` | `grSoftVersion` | `hcl` | `hcl.colors` |
| `hcl.pals` | `heat.colors` | `Hershey` | `hsv` |
| `is.raster` | `jpeg` | `make.rgb` | `n2mfrow` |
| `nclass.FD` | `nclass.scott` | `nclass.Sturges` | `palette` |
| `palette.colors` | `palette.pals` | `pdf` | `pdf.options` |
| `pdfFonts` | `pictex` | `png` | `postscript` |
| `postscriptFonts` | `ps.options` | `quartz` | `quartz.options` |
| `quartz.save` | `quartzFont` | `quartzFonts` | `rainbow` |
| `recordGraphics` | `recordPlot` | `replayPlot` | `rgb` |
| `rgb2hsv` | `savePlot` | `setEPS` | `setGraphicsEventEnv` |
| `setGraphicsEventHandlers` | `setPS` | `svg` | `terrain.colors` |
| `tiff` | `topo.colors` | `trans3d` | `Type1Font` |
| `x11` | `X11` | `X11.options` | `X11Font` |
| `X11Fonts` | `xfig` | `xy.coords` | `xyTable` |
| `xyz.coords` |  |  |  |

### datasets

108 objects: 0 done, 0 partial, 108 missing.

Missing (108):

|   |   |   |   |
| --- | --- | --- | --- |
| `ability.cov` | `airmiles` | `AirPassengers` | `airquality` |
| `anscombe` | `attenu` | `attitude` | `austres` |
| `beaver1` | `beaver2` | `BJsales` | `BJsales.lead` |
| `BOD` | `cars` | `ChickWeight` | `chickwts` |
| `co2` | `CO2` | `crimtab` | `discoveries` |
| `DNase` | `esoph` | `euro` | `euro.cross` |
| `eurodist` | `EuStockMarkets` | `faithful` | `fdeaths` |
| `Formaldehyde` | `freeny` | `freeny.x` | `freeny.y` |
| `gait` | `HairEyeColor` | `Harman23.cor` | `Harman74.cor` |
| `Indometh` | `infert` | `InsectSprays` | `iris` |
| `iris3` | `islands` | `JohnsonJohnson` | `LakeHuron` |
| `ldeaths` | `lh` | `LifeCycleSavings` | `Loblolly` |
| `longley` | `lynx` | `mdeaths` | `morley` |
| `mtcars` | `nhtemp` | `Nile` | `nottem` |
| `npk` | `occupationalStatus` | `Orange` | `OrchardSprays` |
| `penguins` | `penguins_raw` | `PlantGrowth` | `precip` |
| `presidents` | `pressure` | `Puromycin` | `quakes` |
| `randu` | `rivers` | `rock` | `Seatbelts` |
| `sleep` | `stack.loss` | `stack.x` | `stackloss` |
| `state.abb` | `state.area` | `state.center` | `state.division` |
| `state.name` | `state.region` | `state.x77` | `sunspot.m2014` |
| `sunspot.month` | `sunspot.year` | `sunspots` | `swiss` |
| `Theoph` | `Titanic` | `ToothGrowth` | `treering` |
| `trees` | `UCBAdmissions` | `UKDriverDeaths` | `UKgas` |
| `USAccDeaths` | `USArrests` | `UScitiesD` | `USJudgeRatings` |
| `USPersonalExpenditure` | `uspop` | `VADeaths` | `volcano` |
| `warpbreaks` | `women` | `WorldPhones` | `WWWusage` |

### tools

138 objects: 0 done, 0 partial, 138 missing.

Missing (138):

|   |   |   |   |
| --- | --- | --- | --- |
| `.print.via.format` | `add_datalist` | `Adobe_glyphs` | `analyze_license` |
| `as.Rconcordance` | `assertCondition` | `assertError` | `assertWarning` |
| `base_aliases_db` | `base_rdxrefs_db` | `bibstyle` | `buildVignette` |
| `buildVignettes` | `charset_to_Unicode` | `check_package_dois` | `check_package_urls` |
| `check_packages_in_dir` | `check_packages_in_dir_changes` | `check_packages_in_dir_details` | `checkDocFiles` |
| `checkDocStyle` | `checkFF` | `checkMD5sums` | `checkPoFile` |
| `checkPoFiles` | `checkRd` | `checkRdaFiles` | `checkRdContents` |
| `checkReplaceFuns` | `checkS3methods` | `checkSHA256sums` | `checkTnF` |
| `checkVignettes` | `codoc` | `codocClasses` | `codocData` |
| `compactPDF` | `CRAN_aliases_db` | `CRAN_archive_db` | `CRAN_authors_db` |
| `CRAN_check_details` | `CRAN_check_issues` | `CRAN_check_results` | `CRAN_current_db` |
| `CRAN_package_db` | `CRAN_rdxrefs_db` | `delimMatch` | `deparseLatex` |
| `dependsOnPkgs` | `encoded_text_to_latex` | `file_ext` | `file_path_as_absolute` |
| `file_path_sans_ext` | `find_gs_cmd` | `findHTMLlinks` | `followConcordance` |
| `getBibstyle` | `getDepList` | `getVignetteInfo` | `HTMLheader` |
| `installFoundDepends` | `langElts` | `latexToUtf8` | `list_files_with_exts` |
| `list_files_with_type` | `loadPkgRdMacros` | `loadRdMacros` | `make_translations_pkg` |
| `makevars_site` | `makevars_user` | `matchConcordance` | `md5sum` |
| `nonS3methods` | `package_dependencies` | `package_native_routine_registration_skeleton` | `package.dependencies` |
| `parse_Rd` | `parse_URI_reference` | `parseLatex` | `pkg2HTML` |
| `pkgDepends` | `pkgVignettes` | `pskill` | `psnice` |
| `R` | `R_user_dir` | `Rcmd` | `Rd_db` |
| `Rd2ex` | `Rd2HTML` | `Rd2latex` | `Rd2txt` |
| `Rd2txt_options` | `Rdiff` | `Rdindex` | `RdTextFilter` |
| `read.00Index` | `resaveRdaFiles` | `sha256sum` | `showNonASCII` |
| `showNonASCIIfile` | `SIGCHLD` | `SIGCONT` | `SIGHUP` |
| `SIGINT` | `SIGKILL` | `SIGQUIT` | `SIGSTOP` |
| `SIGTERM` | `SIGTSTP` | `SIGUSR1` | `SIGUSR2` |
| `standard_package_names` | `startDynamicHelp` | `summarize_check_packages_in_dir_depends` | `summarize_check_packages_in_dir_results` |
| `summarize_check_packages_in_dir_timings` | `summarize_CRAN_check_status` | `SweaveTeXFilter` | `testInstalledBasic` |
| `testInstalledPackage` | `testInstalledPackages` | `texi2dvi` | `texi2pdf` |
| `toHTML` | `toRd` | `toTitleCase` | `undoc` |
| `update_PACKAGES` | `update_pkg_po` | `verifySHA256signature` | `vignetteDepends` |
| `vignetteEngine` | `vignetteInfo` | `write_PACKAGES` | `xgettext` |
| `xgettext2pot` | `xngettext` |  |  |

### parallel

38 objects: 0 done, 0 partial, 38 missing.

Missing (38):

|   |   |   |   |
| --- | --- | --- | --- |
| `closeNode` | `clusterApply` | `clusterApplyLB` | `clusterCall` |
| `clusterEvalQ` | `clusterExport` | `clusterMap` | `clusterSetRNGStream` |
| `clusterSplit` | `detectCores` | `getDefaultCluster` | `makeCluster` |
| `makeForkCluster` | `makePSOCKcluster` | `mc.reset.stream` | `mcaffinity` |
| `mccollect` | `mclapply` | `mcMap` | `mcmapply` |
| `mcparallel` | `nextRNGStream` | `nextRNGSubStream` | `parApply` |
| `parCapply` | `parLapply` | `parLapplyLB` | `parRapply` |
| `parSapply` | `parSapplyLB` | `pvec` | `recvData` |
| `recvOneData` | `registerClusterType` | `sendData` | `setDefaultCluster` |
| `splitIndices` | `stopCluster` |  |  |

### compiler

9 objects: 0 done, 0 partial, 9 missing.

Missing (9):

|   |   |   |   |
| --- | --- | --- | --- |
| `cmpfile` | `cmpfun` | `compile` | `compilePKGS` |
| `disassemble` | `enableJIT` | `getCompilerOption` | `loadcmp` |
| `setCompilerOptions` |  |  |  |

### grid

249 objects: 0 done, 0 partial, 249 missing.

Missing (249):

|   |   |   |   |
| --- | --- | --- | --- |
| `absolute.size` | `addGrob` | `applyEdit` | `applyEdits` |
| `arcCurvature` | `arrow` | `arrowsGrob` | `as.mask` |
| `as.path` | `ascentDetails` | `bezierGrob` | `bezierPoints` |
| `calcStringMetric` | `childNames` | `circleGrob` | `clipGrob` |
| `convertHeight` | `convertNative` | `convertUnit` | `convertWidth` |
| `convertX` | `convertY` | `current.parent` | `current.rotation` |
| `current.transform` | `current.viewport` | `current.vpPath` | `current.vpTree` |
| `curveGrob` | `dataViewport` | `defineGrob` | `defnRotate` |
| `defnScale` | `defnTranslate` | `delayGrob` | `depth` |
| `descentDetails` | `deviceDim` | `deviceLoc` | `downViewport` |
| `draw.details` | `drawDetails` | `editDetails` | `editGrob` |
| `editViewport` | `emptyCoords` | `emptyGrobCoords` | `emptyGTreeCoords` |
| `engine.display.list` | `explode` | `fillGrob` | `fillStrokeGrob` |
| `forceGrob` | `frameGrob` | `functionGrob` | `gEdit` |
| `gEditList` | `get.gpar` | `getGrob` | `getNames` |
| `gList` | `glyphGrob` | `gpar` | `gPath` |
| `grid.abline` | `grid.add` | `grid.arrows` | `grid.bezier` |
| `grid.cap` | `grid.circle` | `grid.clip` | `grid.collection` |
| `grid.convert` | `grid.convertHeight` | `grid.convertWidth` | `grid.convertX` |
| `grid.convertY` | `grid.copy` | `grid.curve` | `grid.define` |
| `grid.delay` | `grid.display.list` | `grid.DLapply` | `grid.draw` |
| `grid.edit` | `grid.fill` | `grid.fillStroke` | `grid.force` |
| `grid.frame` | `grid.function` | `grid.gedit` | `grid.get` |
| `grid.gget` | `grid.glyph` | `grid.grab` | `grid.grabExpr` |
| `grid.gremove` | `grid.grep` | `grid.grill` | `grid.grob` |
| `grid.group` | `grid.layout` | `grid.legend` | `grid.line.to` |
| `grid.lines` | `grid.locator` | `grid.ls` | `grid.move.to` |
| `grid.multipanel` | `grid.newpage` | `grid.null` | `grid.pack` |
| `grid.panel` | `grid.path` | `grid.place` | `grid.plot.and.legend` |
| `grid.points` | `grid.polygon` | `grid.polyline` | `grid.pretty` |
| `grid.raster` | `grid.record` | `grid.rect` | `grid.refresh` |
| `grid.remove` | `grid.reorder` | `grid.revert` | `grid.roundrect` |
| `grid.segments` | `grid.set` | `grid.show.layout` | `grid.show.viewport` |
| `grid.strip` | `grid.stroke` | `grid.text` | `grid.use` |
| `grid.xaxis` | `grid.xspline` | `grid.yaxis` | `gridCoords` |
| `gridGrobCoords` | `gridGTreeCoords` | `grob` | `grobAscent` |
| `grobCoords` | `grobDescent` | `grobHeight` | `grobName` |
| `grobPathListing` | `grobPoints` | `grobTree` | `grobWidth` |
| `grobX` | `grobY` | `groupFlip` | `groupGrob` |
| `groupRotate` | `groupScale` | `groupShear` | `groupTranslate` |
| `gTree` | `heightDetails` | `is.grob` | `is.unit` |
| `isClosed` | `isEmptyCoords` | `layout.heights` | `layout.torture` |
| `layout.widths` | `layoutRegion` | `legendGrob` | `linearGradient` |
| `linesGrob` | `lineToGrob` | `makeContent` | `makeContext` |
| `moveToGrob` | `nestedListing` | `nullGrob` | `packGrob` |
| `pathGrob` | `pathListing` | `pattern` | `placeGrob` |
| `plotViewport` | `pointsGrob` | `polygonGrob` | `polylineGrob` |
| `pop.viewport` | `popViewport` | `postDrawDetails` | `preDrawDetails` |
| `push.viewport` | `pushViewport` | `radialGradient` | `rasterGrob` |
| `recordGrob` | `rectGrob` | `removeGrob` | `reorderGrob` |
| `resolveHJust` | `resolveRasterSize` | `resolveVJust` | `roundrectGrob` |
| `seekViewport` | `segmentsGrob` | `setChildren` | `setGrob` |
| `showGrob` | `showViewport` | `stringAscent` | `stringDescent` |
| `stringHeight` | `stringWidth` | `strokeGrob` | `textGrob` |
| `unit` | `unit.c` | `unit.length` | `unit.pmax` |
| `unit.pmin` | `unit.psum` | `unit.rep` | `unitType` |
| `upViewport` | `useGrob` | `useRotate` | `useScale` |
| `useTranslate` | `valid.just` | `validDetails` | `viewport` |
| `viewport.layout` | `viewport.transform` | `viewportRotate` | `viewportScale` |
| `viewportTransform` | `viewportTranslate` | `vpList` | `vpPath` |
| `vpStack` | `vpTree` | `widthDetails` | `xaxisGrob` |
| `xDetails` | `xsplineGrob` | `xsplinePoints` | `yaxisGrob` |
| `yDetails` |  |  |  |

### splines

13 objects: 0 done, 0 partial, 13 missing.

Missing (13):

|   |   |   |   |
| --- | --- | --- | --- |
| `as.polySpline` | `asVector` | `backSpline` | `bs` |
| `interpSpline` | `ns` | `periodicSpline` | `polySpline` |
| `spline.des` | `splineDesign` | `splineKnots` | `splineOrder` |
| `xyVector` |  |  |  |

### stats4

28 objects: 0 done, 0 partial, 28 missing.

Missing (13):

|   |   |   |   |
| --- | --- | --- | --- |
| `AIC` | `BIC` | `coef` | `confint` |
| `logLik` | `mle` | `nobs` | `plot` |
| `profile` | `show` | `summary` | `update` |
| `vcov` |  |  |  |

S4 class and generic metadata objects, missing with the S4 system (15): `.__C__mle`, `.__C__profile.mle`, `.__C__summary.mle`, `.__T__AIC:stats`, `.__T__BIC:stats`, `.__T__coef:stats`, `.__T__confint:stats`, `.__T__logLik:stats`, `.__T__nobs:stats`, `.__T__plot:base`, `.__T__profile:stats`, `.__T__show:methods`, `.__T__summary:base`, `.__T__update:stats`, `.__T__vcov:stats`.

### tcltk

272 objects: 0 done, 0 partial, 272 missing.

Missing (272):

|   |   |   |   |
| --- | --- | --- | --- |
| `.Tcl` | `.Tcl.args` | `.Tcl.args.objv` | `.Tcl.callback` |
| `.Tcl.objv` | `.Tk.ID` | `.Tk.newwin` | `.Tk.subwin` |
| `.TkRoot` | `.TkUp` | `addTclPath` | `as.tclObj` |
| `getTkProgressBar` | `is.tclObj` | `is.tkwin` | `setTkProgressBar` |
| `tcl` | `tclArray` | `tclclose` | `tclfile.dir` |
| `tclfile.tail` | `tclObj` | `tclObj<-` | `tclObj<-.tclVar` |
| `tclopen` | `tclputs` | `tclread` | `tclRequire` |
| `tclServiceMode` | `tclvalue` | `tclvalue<-` | `tclvalue<-.default` |
| `tclvalue<-.tclVar` | `tclvar` | `tclVar` | `tclVersion` |
| `tk_choose.dir` | `tk_choose.files` | `tk_messageBox` | `tk_select.list` |
| `tkactivate` | `tkadd` | `tkaddtag` | `tkbbox` |
| `tkbell` | `tkbind` | `tkbindtags` | `tkbutton` |
| `tkcanvas` | `tkcanvasx` | `tkcanvasy` | `tkcget` |
| `tkcheckbutton` | `tkchooseDirectory` | `tkclipboard.append` | `tkclipboard.clear` |
| `tkcompare` | `tkconfigure` | `tkcoords` | `tkcreate` |
| `tkcurselection` | `tkdchars` | `tkdebug` | `tkdelete` |
| `tkdelta` | `tkdeselect` | `tkdestroy` | `tkdialog` |
| `tkdlineinfo` | `tkdtag` | `tkdump` | `tkentry` |
| `tkentrycget` | `tkentryconfigure` | `tkevent.add` | `tkevent.delete` |
| `tkevent.generate` | `tkevent.info` | `tkfind` | `tkflash` |
| `tkfocus` | `tkfont.actual` | `tkfont.configure` | `tkfont.create` |
| `tkfont.delete` | `tkfont.families` | `tkfont.measure` | `tkfont.metrics` |
| `tkfont.names` | `tkfraction` | `tkframe` | `tkget` |
| `tkgetOpenFile` | `tkgetSaveFile` | `tkgettags` | `tkgrab` |
| `tkgrab.current` | `tkgrab.release` | `tkgrab.set` | `tkgrab.status` |
| `tkgrid` | `tkgrid.bbox` | `tkgrid.child` | `tkgrid.columnconfigure` |
| `tkgrid.configure` | `tkgrid.forget` | `tkgrid.info` | `tkgrid.location` |
| `tkgrid.propagate` | `tkgrid.remove` | `tkgrid.rowconfigure` | `tkgrid.size` |
| `tkgrid.slaves` | `tkicursor` | `tkidentify` | `tkimage.create` |
| `tkimage.delete` | `tkimage.height` | `tkimage.inuse` | `tkimage.names` |
| `tkimage.type` | `tkimage.types` | `tkimage.width` | `tkindex` |
| `tkinsert` | `tkinvoke` | `tkitembind` | `tkitemcget` |
| `tkitemconfigure` | `tkitemfocus` | `tkitemlower` | `tkitemraise` |
| `tkitemscale` | `tklabel` | `tklistbox` | `tklower` |
| `tkmark.gravity` | `tkmark.names` | `tkmark.next` | `tkmark.previous` |
| `tkmark.set` | `tkmark.unset` | `tkmenu` | `tkmenubutton` |
| `tkmessage` | `tkmessageBox` | `tkmove` | `tknearest` |
| `tkpack` | `tkpack.child` | `tkpack.configure` | `tkpack.forget` |
| `tkpack.info` | `tkpack.propagate` | `tkpack.slaves` | `tkpager` |
| `tkplace` | `tkplace.child` | `tkplace.configure` | `tkplace.forget` |
| `tkplace.info` | `tkplace.slaves` | `tkpopup` | `tkpost` |
| `tkpostcascade` | `tkpostscript` | `tkProgressBar` | `tkradiobutton` |
| `tkraise` | `tkscale` | `tkscan.dragto` | `tkscan.mark` |
| `tkscrollbar` | `tksearch` | `tksee` | `tkselect` |
| `tkselection.adjust` | `tkselection.anchor` | `tkselection.clear` | `tkselection.from` |
| `tkselection.includes` | `tkselection.present` | `tkselection.range` | `tkselection.set` |
| `tkselection.to` | `tkset` | `tksize` | `tkStartGUI` |
| `tktag.add` | `tktag.bind` | `tktag.cget` | `tktag.configure` |
| `tktag.delete` | `tktag.lower` | `tktag.names` | `tktag.nextrange` |
| `tktag.prevrange` | `tktag.raise` | `tktag.ranges` | `tktag.remove` |
| `tktext` | `tktitle` | `tktitle<-` | `tktoggle` |
| `tktoplevel` | `tktype` | `tkunpost` | `tkwait.variable` |
| `tkwait.visibility` | `tkwait.window` | `tkwidget` | `tkwindow.cget` |
| `tkwindow.configure` | `tkwindow.create` | `tkwindow.names` | `tkwinfo` |
| `tkwm.aspect` | `tkwm.client` | `tkwm.colormapwindows` | `tkwm.command` |
| `tkwm.deiconify` | `tkwm.focusmodel` | `tkwm.frame` | `tkwm.geometry` |
| `tkwm.grid` | `tkwm.group` | `tkwm.iconbitmap` | `tkwm.iconify` |
| `tkwm.iconmask` | `tkwm.iconname` | `tkwm.iconposition` | `tkwm.iconwindow` |
| `tkwm.maxsize` | `tkwm.minsize` | `tkwm.overrideredirect` | `tkwm.positionfrom` |
| `tkwm.protocol` | `tkwm.resizable` | `tkwm.sizefrom` | `tkwm.state` |
| `tkwm.title` | `tkwm.transient` | `tkwm.withdraw` | `tkXselection.clear` |
| `tkXselection.get` | `tkXselection.handle` | `tkXselection.own` | `tkxview` |
| `tkxview.moveto` | `tkxview.scroll` | `tkyposition` | `tkyview` |
| `tkyview.moveto` | `tkyview.scroll` | `ttkbutton` | `ttkcheckbutton` |
| `ttkcombobox` | `ttkentry` | `ttkframe` | `ttklabel` |
| `ttklabelframe` | `ttkmenubutton` | `ttknotebook` | `ttkpanedwindow` |
| `ttkprogressbar` | `ttkradiobutton` | `ttkscale` | `ttkscrollbar` |
| `ttkseparator` | `ttksizegrip` | `ttkspinbox` | `ttktreeview` |

## Names in R-ust that R 4.6.0 does not have

| Name | Notes |
| --- | --- |
| `as.real` | Removed from R in 3.0.0; R-ust treats it as `as.double`. |
| `is.real` | Removed from R in 3.0.0; R-ust treats it as `is.double`. |
| `rnchisq` | Not an R function; R uses `rchisq(n, df, ncp)`. Exposed from the Rmath API. |
| `set_seed` | Not an R function; alias of `set.seed`. |
