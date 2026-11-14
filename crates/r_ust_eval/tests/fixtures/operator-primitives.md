# Arithmetic primitive reference

`operator-primitives-cases.tsv` contains 278 unchanged subject strings and their scopes.
`operator-primitives-r-4.6.0.tsv` is the independent R 4.6.0 semantic gold, SHA-256
`ca122be16dea2628686bc4287a82d26808dec6d522aa601e32b550656961a163`.
The original direct-top-level raw capture fixture has SHA-256
`2284d2cdf45863bcc9c00b7835bd04e54e91a9cafaa94fdefba588a0cbc67740`.
The semantic fixture retains 168 successes and 110 errors, with all 278 subjects in
their original order. The 246 portable rows contain 168 successes and 78 errors.

Run from the repository root with an R 4.6.0 build using 8-byte long double:

```sh
LC_ALL=C R_ENABLE_JIT=0 Rscript --vanilla scripts/operator-primitives-fixture.R \
  crates/r_ust_eval/tests/fixtures/operator-primitives-cases.tsv /tmp/operators-new.tsv
cmp crates/r_ust_eval/tests/fixtures/operator-primitives-r-4.6.0.tsv /tmp/operators-new.tsv
```

The generator rejects an existing output destination before writing. It reads only
the subject table, never the expected semantic fixture. It pins colon's optional
length-check setting to false, evaluates each subject in a fresh base environment,
and records condition messages, raw output bytes, warnings, visibility, primitive
display, and integer/logical or big-endian IEEE-754 fingerprints.

This replay uses `eval`, `withVisible`, warning handlers, and an error catcher as
reference instrumentation. Its condition payloads and effects matched the immutable
gold in two complete 278-row replays. The original 110 direct-top-level error captures
remain authoritative for error payloads; this generator does not claim to reproduce
their CLI call framing or exit statuses.

The Rust test runs all 246 portable subjects through `Frontend` and verifies exact
values, errors, effects, warning payloads, visibility, and handle metadata/display.
The 32 `future_quote` subjects remain explicitly separate. Their Rust SDK mappings
replace only `quote(expr=)` with extraction of an injected `Sexp::MissingValue` leaf.
Those checks cover missing bindings, computed missing bindings/heads/operands, not
public `quote` or `eval` support.

The checkpoint covers genuine first-class handles for `+`, `-`, `*`, `/`, `^`, `%%`,
`%/%`, and `:`, ordinary and optimized call paths, aliases, lazy rebinding, positional
primitive arguments, and the observed shared arithmetic warnings. It does not certify
complex arithmetic, S3/S4 `Ops`, factor colon, all attributes, or all numeric boundaries.

The independent source contract is R 4.6.0 revision 89956: `src/main/names.c` primitive
registrations, `eval.c` callable dispatch/visibility, `envir.c` missing-value function
lookup, `arithmetic.c` arity and numeric warnings, `seq.c` colon length/coercion, and
`util.c` arity wording. Existing numeric/coercion/floor owners remain authoritative.
