Environment value observations captured from GNU R 4.6.0 with `LC_ALL=C`,
`LANG=C`, `R_ENABLE_JIT=0`, `R_DEFAULT_PACKAGES=base`, and `--vanilla --slave`.

The underscore-named print cases and metadata.tsv project the independently
captured environment reference matrix (SHA256
73dea21f4350c7d899d9834b5b726bff43974a5411edbe4dd80781768565fad1).
They preserve the reference printer's output without the final event newline.
The base/namespace count of 1407 documents the full GNU R inventory; native
tests measure the actually installed inventory separately.

The remaining files are separate direct captures, including the final event
newline. Inputs are `globalenv()`, `baseenv()`, `.BaseNamespaceEnv`, `emptyenv()`,
`.AutoloadEnv`, `list(.AutoloadEnv)`,
`list(a=list(b=.AutoloadEnv),z=.AutoloadEnv)`, `list(1L,2L)`,
`list(a=list(b=1L),z=2L)`, and `list(list(),3L)`.

Only hexadecimal pointer tokens inside `<environment: 0x...>` were replaced
by `0xTOKEN`. No attribute text, labels, indentation, or whitespace was removed.
