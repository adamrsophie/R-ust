// R 4.6.0 src/library/base/R/New-Internal.R, GPL-2.0-or-later.
// Copyright (C) The R Core Team.
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        "commandArgs",
        r#"function(trailingOnly = FALSE) {
        args <- .Internal(commandArgs())
        if(trailingOnly) {
            m <- match("--args", args, 0L)
            if(m) args[-seq_len(m)] else character()
        } else args
    }"#,
    ),
    (
        "readline",
        r#"function(prompt = "") .Internal(readline(prompt))"#,
    ),
];
