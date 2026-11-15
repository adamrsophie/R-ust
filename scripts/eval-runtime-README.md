# Eval/evalq frozen-corpus verifier

Run from the repository root:

```
python3 scripts/check-eval-runtime.py self-check
python3 scripts/check-eval-runtime.py baseline PATH/TO/r_ust_cli NEW_OUTPUT
python3 scripts/check-eval-runtime.py compare PATH/TO/r_ust_cli NEW_OUTPUT
```

Python 3.11 or later is required, with no additional Python dependencies. Assertions must remain enabled. The supplied native directory must also contain `Rscript`. GNU R and GNU Rscript must be on PATH and match the pinned runtime. Existing output directories are rejected.

The immutable 74-row file is `crates/r_ust_eval/tests/fixtures/eval-runtime-r-4.6.0.tsv`, SHA256 `d03e9b35b4abb5f7b3bf873c388177d47e7bcdaf27929f8b990b970c016b35ae`. It is an exact copy of `documentation/porting-data/eval-runtime-reference-2026-11-14/final/eval-runtime-r-4.6.0.tsv`. The public source inventory and direct error payload table are also exact copies. The original reference contract and report remain authoritative about measured scope.

There are 52 typed successes and 22 terminal errors. Every ID runs through expression, file, stdin, Rscript-file, and Rscript-expression modes: 370 comparisons, each preserving separate and merged streams. No case is excluded or translated. Setup, subject and postcondition remain distinct top-level lines, followed by the original sentinel. This preserves top-level evalq return and same-environment loop transfers; `eval_base_environment` remains FALSE.

The original typed observer added instrumentation frames. A separate two-pass reference audit evaluated each unchanged successful subject with top-level `.Internal(withVisible(SUBJECT))`, serialized only after evaluation, and checked literal versus internal-observer return/break/next failures. All 52 typed values, metadata, visibility and effects matched the original fixture. All 74 direct subjects matched in every CLI mode in both passes. No fixture expectation was changed. `eval-runtime-observer.R` retains the post-evaluation reference encoder; the native CLI does not execute that encoder.

## Two distinct gates

The CLI gate checks actual formatted payload, printed visibility, ordered effects, warnings, error payloads and exit status. It cannot distinguish hidden type/name/class differences which print identically. The separate `eval_runtime` Rust SDK consumer must check all 74 outcomes, including the 52 exact expected values in `tests/eval_runtime/fixtures.rs`, metadata and visibility, and all 22 terminal errors. Warning classes and calls are retained in the expected rows. Additional promise/control/lifetime scenarios belong to the SDK consumer, not this CLI runner.

`CLI_PASS_TYPED_SDK_REQUIRED` means all 370 CLI comparisons passed and input/runtime hashes stayed stable. It is not complete native parity. `LIMIT` is a failing outcome for an unimplemented native surface. Baseline returns zero after a complete stable capture even when native comparisons fail; its report retains FAIL/LIMIT and exact counts. Compare returns nonzero for native mismatch. Neither action changes or derives expectations from native output. A drifted R reference or incomplete capture fails both actions.

## Schema and comparison

The TSV retains its original 16 columns: ID, category, outcome, node, typeof, class, mode, length, visibility, printed value, warning metadata, effects, status and three raw streams. Byte fields use lowercase hex. Sources are separately bound by the exact 74-row cases TSV. The reader rejects duplicate IDs, missing rows, malformed framing/hex, inconsistent outcome/type/class/mode/length, invalid node structures and any change to frozen source/data/payload hashes. The JSON node decoder is a bounded data-schema reader, not an R parser. Atomic vectors must contain scalar children of their declared type; call/pairlist tags and missing slots are retained. This corpus measures ASCII, unattributed trees and explicit identity predicates, not arbitrary object serialization.

Fresh GNU R output must match the frozen stdout, stderr, merged order and status. Native diagnostics use the existing native error display contract below. Only the complete frozen R diagnostic block is replaced, at its exact position in the merged stream. There is no arbitrary `Calls:` stripping, call-text matching or payload suppression. Extra diagnostic-looking user text, missing warnings/effects, reordered merged output and prefixed/trailing junk fail comparison. Exact hexadecimal environment address tokens are the sole pointer exception; arbitrary hexadecimal text and bytecode pointers remain significant. This corpus uses predicates for callable/environment identity rather than comparing pointer addresses.

| Native prefix | Frozen IDs |
| --- | --- |
| `domain error: ` | primitive_rejected, environment_invalid_enclos, invalid_both, argument_order, evalq_force_order, logical_envir, character_envir, numeric_length, numeric_na, numeric_range, partial_escalated |
| `argument error: ` | unused_arg, ambiguous_arg |
| `control-flow error: ` | break_outside, break_other_environment |
| No prefix | missing_expression, missing_evalq, computed_missing, bound_missing, omitted_forward, expr_force_first, expression_error |

Warnings use exactly `Warning: MESSAGE` plus a newline. These are source-declared native envelopes, not a claim of byte-identical GNU R framing or hidden condition-object parity. Native error calls/classes and warning metadata require their SDK checks.

## Capture integrity

The verifier reuses the established capture/invocation helper. It snapshots both native executables, records exact argv/stdin and relevant source/program/binary hashes, and retains stdout, stderr, merged output and both process statuses. Subject runs use an explicitly recorded environment consisting of the pinned locale/JIT/base-package settings and only the required host path, home and temporary-directory variables. R version, launcher, executable and libR hashes are checked. Inputs, source membership, generated programs, snapshots and runtime hashes are checked again after the complete run. Output artifacts receive a final hash manifest.

These records do not establish compilation binding of a supplied binary to the current checkout, transitive runtime-library identity, bytecode execution, S4 environment conversion or unmeasured stack depths. Those remain separate integration or porting work.
