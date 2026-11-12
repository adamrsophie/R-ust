# R-ust

R-ust is an implementation of the R language in Rust, targeting compatibility with R 4.6.0.

[Project website](https://adamrsophie.github.io/R-ust/)

The project is an early proof of concept. It provides a pure Rust port of R's standalone mathematical library (Rmath), a C ABI for that library, and a parser, evaluator, and command line interface that run a growing subset of the R language.

## Quick start

R-ust requires a stable Rust toolchain (edition 2021).

```bash
cargo build --release
```

Start the interactive console:

```bash
cargo run -q -p r_ust_cli
```

The console keeps variables and functions between commands. Use multiple lines for unfinished expressions; `+ ` marks continuation input. Errors return to the prompt. Use `q()` or Ctrl-D to exit, then choose `n` to discard the workspace or `c` to cancel. Workspace saving is not implemented; choosing `y` reports an error and keeps the console open. Add `-- --quiet --no-save` to suppress the startup banner and default save prompt.

The main CLI echoes input by default, including scripts, expressions and piped commands. `--quiet` or `--silent` hides the startup banner. `--no-echo`, `-s` or `--slave` suppresses input echo and prompts, hides the banner, and implies `--no-save`. Automatic printing of visible values continues. `options(echo = FALSE)` suppresses subsequent input echo and prompts; `TRUE` or `NA` enables them on the next read. Rscript starts with echo disabled.

Ask for an answer inside the terminal console:

```r
name <- readline("Name: ")
cat("Hello,", name, "\n")
```

`readline()` flushes its prompt before reading an answer, trims leading and trailing spaces and tabs, and returns a visible character scalar. EOF at the question returns `""`. In expressions, scripts, and piped input, it prints the prompt with a newline and returns `""` without consuming stdin. Compound prompt deparsing, byte truncation that would produce invalid UTF-8, and existing number-to-string differences remain limitations.

Configure console prompts and restore previous settings:

```r
old <- options(prompt = "R> ", continue = "more> ")
getOption("prompt")
options(old)
```

Each runtime owns its option store. `options()` returns a snapshot; named arguments or a single named list set values and return previous values invisibly. Unnamed character arguments query options. Setting a custom option to `NULL` removes it. `getOption(name, default)` evaluates the default only when the option is unset. Core setters validate and coerce values using R's bounds and type rules; earlier changes survive a later invalid setter in the same call. Primary and continuation prompt changes take effect on the next console read, including after an error.

Option support remains partial. The store initializes core defaults; package, platform, and startup-file defaults are incomplete. Printing, evaluation-depth limits, source retention, warning conditions, and other option consumers still need work. `.Options` pairlists, locale-sensitive ordering of option names, and missing names in query-result lists are not supported. Setter warnings use direct stderr output, without R condition objects or full diagnostic formatting.

Complete statements run before later input is parsed. Earlier output survives a later syntax error, and an incomplete statement waits for more input without repeating commands already executed.

`q("no", status = 3)` exits with the requested process status. A global `.Last` function runs on normal exit unless `runLast = FALSE`. Expressions, scripts, and piped input default to exiting without saving.

Evaluate an expression:

```bash
cargo run -q -p r_ust_cli -- --no-echo -e 's <- 0; for (i in 1:5) { if (i == 3) next; s <- s + i }; s; pnorm(qnorm(0.7))'
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
cargo run -q -p r_ust_cli -- --no-echo example.R
```

```text
[1] "review" "pass" "pass" "review" "review"
[1] 79
```

Read commands from stdin, without prompts:

```bash
printf 'x <- 1:5\nsum(x)\n' | cargo run -q -p r_ust_cli -- --no-echo
```

Use `-` or `-f -` to request stdin explicitly, and `--help` for available options. Script and piped input stop on the first error with a nonzero exit status.

Pass arguments after a script path, or use `--args` after expressions or `-f`/`--file=FILE`:

```bash
cargo run -q -p r_ust_cli -- --no-echo -e 'writeLines(commandArgs(TRUE))' --args first "two words"
cargo run -q -p r_ust_cli -- --no-echo example.R first "two words"
```

`commandArgs()` returns the invocation, including the program name. For a positional script, an implicit `--args` separator is inserted before its arguments. `commandArgs(trailingOnly = TRUE)` returns only arguments after the first separator, including empty strings and option-like values. `interactive()` is true only for the terminal console, and false for expressions, scripts, and explicit stdin.

Repeat `-e` to evaluate expressions in one session. Each admitted expression gets a terminating newline, so comments and unfinished syntax can continue across `-e` arguments. Combining an admitted expression with a file option is an error. Echo uses primary or continuation prompts at each input read, includes comments and blank lines, normalizes CRLF endings, and stops reading source on quit or error. Code after a semicolon on an already-read line can therefore appear in echo even when it is never evaluated.

Expressions share R's 10,000-byte input buffer. Admission counts existing decoded source, the new encoded argument and two terminator bytes. Oversized arguments are omitted with a startup warning; later arguments can still fit. Omitted arguments remain visible in `commandArgs()`. If none are admitted, input falls back to non-interactive stdin. The main CLI can also run an explicit file alongside omitted expressions. Startup warnings precede evaluation and preserve macOS R's byte truncation, even when it cuts inside UTF-8. `scripts/check-expression-buffer.py` compares these cases with R 4.6.0.

Both frontends process `-e` backslashes and whitespace markers using the macOS R 4.6.0 launcher's rules, including octal/hex byte escapes, escaped backslashes, and NUL removal. `commandArgs()` exposes the encoded expression; execution receives decoded source. `scripts/check-cli-launcher.py` compares source execution and invocation encoding with R. Other shells' echo rules are unverified. The launcher rejects `\c` truncation and non-UTF-8 escape results explicitly. Quoted strings decode R control, octal/hex, and `\u`/`\U` escapes; non-UTF-8 strings and unpaired surrogates remain unsupported.

Remaining input boundaries: lines longer than R's console buffer echo as one line rather than separate buffer chunks. Stdin encoding translation and byte cuts inside UTF-8 characters are not implemented. `scripts/check-cli-echo.py` records long-line differences separately from passing comparisons. Main CLI argument errors return status 2, including invalid `-e` values; R's launcher uses status 1.

## Native Rscript frontend

The workspace also builds a native `Rscript` executable. It uses the Rust interpreter directly and does not require an R installation.

```bash
cargo build --release -p r_ust_cli --bins
./target/release/Rscript --vanilla -e 'sum(1:5)'
./target/release/Rscript --vanilla example.R first "two words"
cargo run -q -p r_ust_cli --bin Rscript -- example.R first "two words"
```

Rscript runs files, repeated `-e` expressions, and stdin via `-`. It is non-interactive even when stdin is a terminal. After a file, every remaining token is a user argument. After expressions, only consecutive `-e` options add code; the first other token starts user arguments. Put frontend options before the file or first expression. A literal `--args` after code is itself a user argument, matching Rscript.

Script files preserve their trailing blank lines and EOF behavior. Rscript reads a file once and evaluates it without the extra newline used for `-e` arguments. `scripts/check-file-input.py` compares files and piped input with R, including LF/CRLF, comments, errors, quit handling, and whitespace rules.

`commandArgs()` includes the native executable name, the default `--no-echo` and `--no-restore` options, and the normalized invocation. Expression whitespace and file-path spaces use the Unix R launcher's encoding. `commandArgs(TRUE)` preserves user arguments, including empty strings. Source-file open failures return status 2; parse and evaluation errors return status 1; `q()` preserves its requested status.

For executable scripts, put the directory containing R-ust's `Rscript` on `PATH` and start the script with `#!/usr/bin/env Rscript`. Joined shebang options such as `--vanilla --quiet` are supported. `--help`, `--version`, and `--verbose` describe this R-ust frontend. Package selection, startup file loading, workspace saving/restoration, and other R options remain incomplete; requests for unavailable package selection or save/restore behavior fail explicitly.

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

Embedded evaluators report `EvalError::Quit(QuitRequest)` without exiting the host process. The CLI handles shutdown and returns the requested integer process status from `r_ust_cli::run`.

`CliConfig::startup_stdout` holds raw bytes emitted before the session starts. Struct literals must initialize it or use `..CliConfig::default()`. `run_to_string` captures UTF-8 startup output and returns an error when those bytes are not valid UTF-8; `run` streams the original bytes.

Hosts can set R-visible arguments and interactive status through `RuntimeState::set_session_context`. Embedded runtimes default to an empty argument vector and non-interactive status; they do not expose the host process's arguments automatically.

`RuntimeState::get_option` reads the host's option store without changing visibility. `RuntimeState::set_option` applies the same validation as R-visible setters and returns the previous value. Setting session context initializes the `echo` and `keep.source` options for that session mode; CLI startup also records its `quiet` setting.

Interactive embedded hosts install a console reader through `RuntimeState::set_input_reader`; the callback returns one raw line or an empty string at EOF. The evaluator never reads host stdin automatically. Input failures propagate as `EvalError::Input`, and output failures prevent input from being read.

## Supported language features

**Syntax and evaluation**

- Double, integer (`1L`), logical, character, and `NULL` literals
- R's UTF-8 blank class plus form feed, with LF and CRLF line endings. Bare CR, vertical tab, and nonbreaking spaces are not whitespace separators.
- Single- and double-quoted strings with R control escapes, octal/hex bytes, Unicode escapes (including braced forms and paired surrogates), and escaped newlines; unknown escapes and mixed Unicode/byte escapes are errors
- Raw string literals with `r` or `R`, either quote, `()`, `[]`, `{}`, or `||` delimiters, and matching dash counts; backslashes remain literal
- Backtick-quoted names in assignments, calls, parameters, argument tags, loops, and `$` access; control and UTF-8 byte escapes decode, while Unicode escapes inside backticks are errors
- Assignment with `<-`, `=`, `->`, `<<-`, and `->>`, including replacement through `[`, `[[`, `$`, and `names<-`
- Control flow: `if`/`else`, `for`, `while`, `repeat`, `break`, `next`, and `return()`
- Closures with lazily evaluated arguments and defaults, `missing()`, `...` forwarding, and exact named and positional argument matching
- Operators: arithmetic, comparison, `:`, `&`, `|`, `&&`, `||`, and `%in%`
- Result visibility through assignments, calls, parentheses, `invisible()`, and `withVisible()`

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
- Console output: `print()`, `cat()`, and default `writeLines()` output on stdout, and `message()` output on stderr, including inside functions and loops

## Rmath library

`r_ust_nmath` implements all 154 public functions of R's standalone `Rmath.h` interface in pure Rust. `r_ust_ffi` exports the same functions through a C ABI, built as a static and dynamic library, so existing C code written against Rmath can link against R-ust.

## Known limitations

R-ust does not yet provide reference semantics for environments, general replacement functions, full attribute support (including row names and arrays), formulas, S3 and S4 dispatch, namespaces and package loading, condition handling, connections, graphics, serialization, or the base and recommended packages.

String values and symbol names require UTF-8. Non-UTF-8 byte strings/names, unpaired surrogates, encoding metadata, and full R display formatting remain incomplete. `scripts/check-string-escapes.py`, `scripts/check-raw-strings.py`, and `scripts/check-backticks.py` compare string/name values and errors through stdin, files, and both expression frontends with R 4.6.0. Calling operators or special forms by quoted name remains incomplete.

The console uses basic terminal input. Command history, advanced line editing, interrupt recovery, workspace saving, `.Last.sys`, exit finalizers, and full R command line compatibility remain missing. Quit warnings print directly to stderr without R condition objects or full diagnostic formatting.

Output formatting differs from R in places: numbers generally use 12 decimal places with trailing zeros trimmed, while tiny values use 12 significant digits. R defaults to 7 significant digits. Names are not yet shown when printing named vectors, and character vectors are not column aligned.

Console output flushes during evaluation. Advanced printing options and S3 methods, file connections, `cat()` filling and vector separators, and message conditions and translation domains remain incomplete.

Several Rmath families use different algorithms from R, and their agreement with R has not yet been validated across the full parameter range:

- Bessel functions use series, reflection, and asymptotic approximations in place of R's Cody and Amos style routines.
- Noncentral beta, F, t, and chi-square distributions use mixture, integration, and bisection methods.
- The Tukey studentized range distribution and quantile use a monotone approximation in place of R's quadrature algorithm.
- Random number generators for the gamma, beta, F, t, binomial, Poisson, hypergeometric, rank, and multinomial families do not reproduce R's random streams.
- Wilcoxon and signed rank distributions use exact dynamic programming tables; the exported `*_free` functions are no-ops.

## Testing

```bash
cargo test
scripts/check-c-abi.sh
python3 scripts/check-cli-quit.py
python3 scripts/check-cli-session.py
python3 scripts/check-rscript.py
python3 scripts/check-cli-readline.py
```

`scripts/check-c-abi.sh` builds the static library and links a C program against it to exercise the exported Rmath ABI. It requires a C compiler.

`scripts/check-cli-quit.py` compares exit behavior with installed R using isolated processes. It requires `Rscript` and a built CLI, records results in `target/cli-quit-parity.json`, and reports unsupported workspace saving separately. Error wording and full warning layout are not compared.

`scripts/check-cli-session.py` compares argument handling, session metadata, expression sequencing, and condition checks with installed R. It requires `R` and a built CLI and records results in `target/cli-session-parity.json`. Diagnostic wording is not compared.

`scripts/check-rscript.py` compares the native frontend with an independent Rscript 4.6.0 installation on `PATH`. It snapshots the built executable, checks argument normalization, scripts, expressions, stdin, and exit behavior, and records results in `target/rscript-parity.json`. Unsupported package/save/restore options are reported separately. Diagnostic, help, version, and verbose wording are not compared.

`scripts/check-cli-readline.py` requires Unix PTYs, independent R/Rscript 4.6.0 references, and both built executables. It snapshots the executables and checks batch output, live prompt readiness, buffered answers, EOF, save cancellation, and input inside `.Last`. Results and lossless stream captures are recorded in `target/cli-readline-parity.json`; known coercion and encoding limits are excluded from passing coverage. Diagnostic wording is not compared.

## License

R-ust is free software, distributed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).
