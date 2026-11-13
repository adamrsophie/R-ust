use r_ust_sexp::{CallArgument, InternalOperation, Sexp};

// R 4.6.0 src/library/base/R/rle.R, Copyright (C) 1995-2014 The R Core Team.
// Part of the R package, https://www.R-project.org
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// A copy of the GNU General Public License is available at
// https://www.R-project.org/Licenses/
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        "rle",
        r#"function(x)
{
    if (!is.vector(x) && !is.list(x))
        stop("'x' must be a vector of an atomic type")
    n <- length(x)
    if (n == 0L)
        return(structure(list(lengths = integer(), values = x),
                         class = "rle"))
    y <- x[-1L] != x[-n]
    i <- c(which(y | is.na(y)), n)
    structure(list(lengths = diff(c(0L, i)), values = x[i]),
              class = "rle")
}"#,
    ),
    (
        "inverse.rle",
        r#"function(x, ...)
{
    if(is.null(le <- x$lengths) ||
       is.null(v  <- x$values) || length(le) != length(v))
        stop("invalid 'rle' structure")
    rep.int(v, le)
}"#,
    ),
];

pub(super) fn lower(name: &str) -> Option<Sexp> {
    let operation = match name {
        "rle" => InternalOperation::RunLengthEncode,
        "inverse.rle" => InternalOperation::RunLengthDecode,
        _ => return None,
    };
    Some(Sexp::InternalCall {
        operation,
        arguments: vec![CallArgument::positional(Sexp::Symbol("x".into()))],
    })
}
