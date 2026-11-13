// R 4.6.0 src/library/base/R/rowsum.R: Copyright (C) 1995-2023 The R Core Team
// Part of the R package, https://www.R-project.org
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// A copy of the GNU General Public License is available at
// https://www.R-project.org/Licenses/
// Adapted bodies: represented data-frame/default dispatch and compiler-private helpers.
// General S3 dispatch and row-name attributes remain unsupported.
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    ("rowsum", "function(x, group, reorder = TRUE, ...) if(is.data.frame(x)) rowsum.data.frame(x, group, reorder, ...) else rowsum.default(x, group, reorder, ...)"),
    ("rowsum.default", r#"function(x, group, reorder = TRUE, na.rm = FALSE, ...) { if(!is.numeric(x)) stop("'x' must be numeric"); .rowsum.check(x, group); if(reorder) reorder <- TRUE else reorder <- FALSE; .rowsum.sum(x, group, reorder, na.rm) }"#),
    ("rowsum.data.frame", r#"function(x, group, reorder = TRUE, na.rm = FALSE, ...) { if(!is.data.frame(x)) stop("not a data frame"); .rowsum.check(x, group); if(reorder) reorder <- TRUE else reorder <- FALSE; .rowsum.sum(x, group, reorder, na.rm) }"#),
];

pub(super) fn resolve(name: &str) -> Option<r_ust_sexp::InternalOperation> {
    match name {
        ".rowsum.check" => Some(r_ust_sexp::InternalOperation::RowsumValidate),
        ".rowsum.sum" => Some(r_ust_sexp::InternalOperation::RowsumAggregate),
        _ => None,
    }
}
