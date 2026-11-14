# Environment reference observations

Run from the repository root with GNU R 4.6.0 available as `R` and `Rscript`:

```sh
python3 scripts/check-environment-values.py reference NEW_OUTPUT
python3 scripts/check-environment-values.py self-check
```

The reference command accepts no native binary. It creates a new directory and refuses
an existing destination or optimized Python (`-O`). It requires the recorded
`aarch64-apple-darwin23` profile, C locale, base-only startup, and JIT disabled.

The frozen fixture SHA-256 is
`3cda1fa2849f25dce776684092d62d4d85700e335f90546fdc89db2c164414d3`.
Its original 16-column typed/direct table reconstructs SHA-256
`73dea21f4350c7d899d9834b5b726bff43974a5411edbe4dd80781768565fad1`.
The command reads these expectations; it never replaces them.

`scripts/environment-values-cases.tsv` supplies the original setup, subject and after
expression boundaries. These are input programs, with no expected values. Its ordered
128 IDs, categories, expected terminal statuses, and assembled source bytes must match
the frozen fixture. The input file SHA-256 is
`85f97aecfb76fc0bb2ee57c2aa8c4746b4879211df8e9d1aec7d348d4171c341`.

Two independent iterations each launch separate-stream, merged-stream and typed R
processes for every row. The public R observer captures values, type, class, mode,
length, visibility, printing, warnings and ordered effects. Error rows retain the
original empty value fields and independently observe the condition message, including
setup failures. Terminal status and complete diagnostics remain in the direct captures.
The 97 CLI rows also run in expression, file, stdin, Rscript-file and Rscript-expression
modes, each with separate and merged streams. All 28 SDK and three reference-only rows
remain in the reference capture; this does not establish native SDK, lifetime or GC parity.

Every raw record stores the invocation, status and full streams, including base64 bytes.
Only hexadecimal pointer tokens inside `<environment: 0x...>` markers are excluded
from comparison. Autoloads attributes, indentation, blank lines, diagnostics and all
other bytes remain significant. Pointer tokens cannot prove environment identity.

`verification.json` records ordered membership and outcomes. `integrity.json` binds
the public sources, input programs, R launchers, actual executable, libR and raw captures
with before/after SHA-256 manifests. Transitive runtime libraries are not bound.
Failed attempts retain their captured records and are never reused as expectations.
