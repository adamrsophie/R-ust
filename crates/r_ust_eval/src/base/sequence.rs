// R 4.6.0 src/library/base/R/seq.R, GPL-2.0-or-later.
// Copyright (C) 1995-2025 The R Core Team.

pub(super) const GENERIC: &str = r#"function(nvec, ...) UseMethod("sequence")"#;
pub(super) const DEFAULT: &str = r#"function(nvec, from = 1L, by = 1L,
                             recycle = Sys.getenv("R_sequence_recycle", "false"), ...) {
    .Internal(sequence(as.integer(nvec), as.integer(from), as.integer(by),
                       if(missing(recycle)) 0L else as.logical(recycle)))
}"#;
