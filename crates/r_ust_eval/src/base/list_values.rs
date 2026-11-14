// Adapted from R 4.6.0 src/library/base/R/structure.R and pairlist.R.
// Copyright (C) 1995-2025 The R Core Team
// SPDX-License-Identifier: GPL-2.0-or-later

pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        "as.pairlist",
        r#"function(x) .Internal(as.vector(x, "pairlist"))"#,
    ),
    ("pairlist", "function(...) as.pairlist(list(...))"),
    (
        "structure",
        r#"function (.Data, ...)
{
    if(is.null(.Data))
        stop("attempt to set an attribute on NULL")
    attrib <- list(...)
    if(length(attrib)) {
        specials <- c(".Dim", ".Dimnames", ".Names", ".Tsp", ".Label")
        attrnames <- names(attrib)
        m <- match(attrnames, specials)
        ok <- !is.na(m)
        if(any(ok)) {
            replace <- c("dim", "dimnames", "names", "tsp", "levels")
            names(attrib)[ok] <- replace[m[ok]]
        }
        ## prior to 2.5.0 factors would deparse to double codes
        if(any(attrib[["class", exact = TRUE]] == "factor")
           && typeof(.Data) == "double")
            storage.mode(.Data) <- "integer"
        attributes(.Data) <- c(attributes(.Data), attrib)
    }
    .Data
}"#,
    ),
];
