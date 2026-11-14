# Canonical language checkpoint 1 reference data

These tables were captured independently from R 4.6.0 with source retention disabled. They are not generated from R-ust output.

`scripts/canonical-language-oracle.R` emits the original 102 tab-separated records: 48 metadata, 29 projections, 20 equalities, and five runtime controls. Run `Rscript --vanilla scripts/canonical-language-oracle.R`; an optional case ID selects one case. `--profile` prints the R/platform/locale profile. The fixture source columns preserve the independently authored parser input; projection paths use one-based language cells, including the head, and pairlist elements. `M`, `P`, `E`, and `R` prefixes identify the corresponding table rows.

`scripts/canonical-language-assignment-oracle.R` emits seven additional independent assignment comparisons, covering ordinary, subset, matrix subset, extraction, both dollar member kinds, and names replacement assignments. All differ between `=` and `<-`. Run it with `Rscript --vanilla` to replay `assignment-equalities.tsv`.

`scripts/canonical-language-nested-identity-oracle.R` emits four further independent comparisons of nested infix/prefix calls, defaults containing infix/prefix calls, and NaN versus NA defaults. Its output reproduces `nested-identity-equalities.tsv`.

`scripts/canonical-language-replacement-oracle.R` compares six replacement assignments with explicitly spelled language calls. `replacement-equalities.tsv` verifies that their left sides remain extraction calls rather than replacement-function calls.

`scripts/canonical-language-mode-oracle.R` reproduces the 11 `modes.tsv` rows, including the distinction between type `symbol` and mode `name`, and type `language` and mode `call`. A `-` projection denotes the whole value.

The foundation exposes `r_ust_sexp::canonical_language`, `LanguageError`, `AssignmentOperator`, `MemberName`, `PairList`, and `Sexp::pairlist`. Assignment AST struct literals now require an `operator` field for all six ordinary/replacement variants. Dollar fields use `MemberName::Symbol` or `MemberName::String`, with `as_str()` for execution. Empty pairlists normalize to `Sexp::Nil`; the `PairList` field is private. `Sexp::MissingValue` is canonical missing syntax and remains distinct from named diagnostic `MissingArg` values.

Baseline dollar parsing rejected string literals; this checkpoint narrowly adds a dedicated member parser. It does not broaden formal/name parsing. Canonical conversion rejects internal dispatch and runtime containers/handles explicitly. Storage may still contain graph-managed closures and promises beneath Calls/PairLists. Existing numeric/coercion and diagnostic paths retain their established errors or unsupported behavior for the new types. Metadata, semantic identity, and `is.list` are supported; general pairlist operations are not added here.

No public `quote`, `eval`, language printer, attributes, source-reference retention, improper pairlists, or arbitrary mutable cons cycles are implemented by this checkpoint.
