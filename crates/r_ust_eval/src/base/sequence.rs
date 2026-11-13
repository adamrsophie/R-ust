// R 4.6.0 src/library/base/R/seq.R, GPL-2.0-or-later.
// Copyright (C) 1995-2025 The R Core Team.
use crate::{CallArgument, Sexp};

pub(super) const GENERIC: &str = "function(nvec, ...) UseMethod(\"sequence\")";
pub(super) const DEFAULT: &str = r#"function(nvec, from = 1L, by = 1L,
    recycle = Sys.getenv("R_sequence_recycle", "false"), ...) {
    .Internal(sequence(as.integer(nvec), as.integer(from), as.integer(by),
        if(missing(recycle)) 0L else as.logical(recycle)))
}"#;

pub(super) fn lower_generic(body: &mut Sexp) {
    // shortcut: default-only UseMethod lowering, replace when S3 dispatch is implemented.
    let call = Sexp::Call {
        function: Box::new(Sexp::Symbol("sequence.default".into())),
        arguments: vec![
            CallArgument::positional(Sexp::Symbol("nvec".into())),
            CallArgument::positional(Sexp::Symbol("...".into())),
        ],
    };
    *body = Sexp::Block(vec![Sexp::Symbol("nvec".into()), call]);
}
