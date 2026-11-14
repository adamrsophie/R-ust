# Missing-value runtime reference

`missing-value-runtime-r-4.6.0.tsv` contains 16 independent R 4.6.0 executions. Fields are case name, R source, process exit status, stdout hex, and stderr hex (`-` means empty). Errors execute directly at top level, without condition handlers. The generator captures each source, both raw streams, exit status, an exact replay, the R version, and a SHA-256 manifest:

```
uv run scripts/missing-value-runtime-reference.py OUTPUT_DIRECTORY
```

Copy the resulting `reference.tsv` to this fixture only after reviewing the independent R results. R source owners are R 4.6.0 `src/main/eval.c` ordinary symbol lookup, `src/main/envir.c` `findFun3`, `R_isMissing`, and `R_missing`, and `src/main/match.c` argument matching.

SDK tests initialize a literal list containing `MissingValue` and replace the R oracle's `quote(expr=)` expression with extraction of that element. Extraction evaluates to the same canonical leaf, so it tests computed promise results without registering quote. A raw `MissingValue` call argument is tested separately because it represents an explicit omitted argument, unlike the supplied expression that computes missing.

The fixture checks exact effect streams and missing-argument payloads. Native error call-stack framing and quote parsing/printing are outside this SDK comparison. Public quote, eval, and canonical printing remain unavailable. Raw discovery captures are private evidence, not fixture contents.
