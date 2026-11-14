// R 4.6.0 src/library/base/R/seq.R, GPL-2.0-or-later.
// Copyright (C) 1995-2025 The R Core Team.
use crate::Sexp;

pub(super) const GENERIC: &str = "function(nvec, ...) UseMethod(\"sequence\")";
pub(super) const DEFAULT: &str = r#"function(nvec, from = 1L, by = 1L,
    recycle = Sys.getenv("R_sequence_recycle", "false"), ...) {
    .Internal(sequence(as.integer(nvec), as.integer(from), as.integer(by),
        if(missing(recycle)) 0L else as.logical(recycle)))
}"#;

pub(super) fn lower_generic(body: &mut Sexp) {
    // shortcut: default-only dispatch, replace when genuine S3 dispatch is implemented.
    *body = Sexp::InternalCall {
        operation: r_ust_sexp::InternalOperation::SequenceDispatch,
        arguments: Vec::new(),
    };
}
