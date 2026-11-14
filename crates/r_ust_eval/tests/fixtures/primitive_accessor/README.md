# Primitive accessor reference

The independent R 4.6.0 fixture has 62 rows (41 typed values and 21 terminal errors), SHA-256 `9385faec9f8c0963cde7dbc7c4fd8030f604b319b09ad826e9169fda644737db`.

All rows remain in `mapping.tsv`: 54 direct-source cases; four positive R catalogue entries (`if`, `sqrt`, `[`, `.Internal`) whose handles are not represented by this runtime; and four reference-only helper/attribute cases using unported `formals`, `args`, `structure`, or `warning`. The catalogue checks require either the correct represented handle or an explicit unsupported-catalogue error, never the false claim that these R primitives do not exist. They allow later ports without pinning permanent absence. These are boundary checks, not four native parity passes. No complete catalogue/accessor or 62-row native parity claim is made.

Primary owners are `names.c:119/1063/1075`, `eval.c:1248`, `util.c:516`, and base `zzz.R:77` in R 4.6.0. `.Primitive` itself is a genuine builtin primitive. It evaluates arguments before count/type validation, ignores tags, requires one scalar string, and returns the same represented identity. NA string reaches unknown-name lookup. Unknown and internal-only names error; known public but unported names are explicitly unsupported. Printed signature metadata remains `function (name)  .Primitive(".Primitive")` without implementing the reference-only args/formals helpers.

The public catalogue table is shared with the canonical printer and exactly follows the 210 non-internal entries in pinned R `names.c`, preserving table order and bytes. The native decoder consumes actual R trees. It does not manufacture expected trees through the Rust parser.

Use `LC_ALL=C LANG=C R_ENABLE_JIT=0 R_DEFAULT_PACKAGES=base R --vanilla --slave -f generate.R --args cases.tsv NEW_CAPTURE`. The generator refuses existing output and incompatible R/profile settings. Its independent observer preserves genuine primitive identity, metadata, visibility, print text, effects and warnings. Terminal conditions come from direct processes, not wrapped substitutes. Native CLI tests compare exact stdout/status and the independent error payloads with the existing native diagnostic framing.

With the same environment, `R --vanilla --slave -f ../quote_runtime/verify.R --args FIXTURE_DIRECTORY NEW_CAPTURE` verifies byte-exact fixture reproduction plus repeated direct/typed captures. The shared `../quote_runtime/check-guards.R` accepts this fixture directory and a new guard-output directory to check output preservation and five incompatible profiles.
