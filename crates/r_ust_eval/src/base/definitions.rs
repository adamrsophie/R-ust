// R 4.6.0 src/library/base/R/stop.R: Copyright (C) 1995-2025 The R Core Team
// R 4.6.0 src/library/base/R/message.R: Copyright (C) 1995-2021 The R Core Team
// R 4.6.0 src/library/base/R/New-Internal.R: Copyright (C) 1995-2023 The R Core Team
// R 4.6.0 src/library/base/R/vector.R: Copyright (C) 1995-2013 The R Core Team
// R 4.6.0 src/library/base/R/constants.R: Copyright (C) 1995-2012 The R Core Team
// R 4.6.0 src/library/base/R/match.R: Copyright (C) 1995-2025 The R Core Team
// R 4.6.0 src/library/base/R/character.R: Copyright (C) 1995-2025 The R Core Team
// R 4.6.0 src/library/base/R/raw.R: Copyright (C) 1995-2020 The R Core Team
// R 4.6.0 src/library/base/R/findInt.R: Copyright (C) 1995-2024 The R Core Team
// R 4.6.0 src/library/base/R/sets.R: Copyright (C) 1995-2025 The R Core Team
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
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    // shortcut: set coercion is lowered into existing match handling for atomic,
    // factor and matrix values; extend with list matching and S3 support.
    (
        "is.element",
        "function(el, set) { x <- el; y <- set; match(x, y, 0L) > 0L }",
    ),
    ("utf8ToInt", "function(x) .Internal(utf8ToInt(x))"),
    ("intToUtf8", "function(x, multiple = FALSE, allow_surrogate_pairs = FALSE) .Internal(intToUtf8(x, multiple, allow_surrogate_pairs))"),
    (
        "findInterval",
        r#"function(x, vec, rightmost.closed = FALSE, all.inside = FALSE,
                   left.open = FALSE, checkSorted = TRUE, checkNA = TRUE) {
    if(checkSorted && !identical(FALSE, is.unsorted(vec)))
        stop("'vec' must be sorted non-decreasingly and not contain NAs")
    .Internal(findInterval(as.double(vec), as.double(x),
                           rightmost.closed, all.inside, left.open, checkNA))
}"#,
    ),
    (
        "strtoi",
        "function(x, base = 0L) .Internal(strtoi(as.character(x), as.integer(base)))",
    ),
    (
        "pmatch",
        "function(x, table, nomatch = NA_integer_, duplicates.ok = FALSE) .Internal(pmatch(as.character(x), as.character(table), nomatch, duplicates.ok))",
    ),
    (
        "charmatch",
        "function(x, table, nomatch = NA_integer_) .Internal(charmatch(as.character(x), as.character(table), nomatch))",
    ),
    (
        "vector",
        "function(mode = \"logical\", length = 0L) .Internal(vector(mode, length))",
    ),
    (
        "logical",
        "function(length = 0L) .Internal(vector(\"logical\", length))",
    ),
    (
        "integer",
        "function(length = 0L) .Internal(vector(\"integer\", length))",
    ),
    (
        "double",
        "function(length = 0L) .Internal(vector(\"double\", length))",
    ),
    (
        "numeric",
        "function(length = 0L) .Internal(vector(\"double\", length))",
    ),
    (
        "character",
        "function(length = 0L) .Internal(vector(\"character\", length))",
    ),
    (
        "stop",
        r#"function(..., call. = TRUE, domain = NULL)
{
    if (...length() == 1L && inherits(..1, "condition")) {
        cond <- ..1
        if(nargs() > 1L)
            warning("additional arguments ignored in stop()")
        message <- conditionMessage(cond)
        call <- conditionCall(cond)
        .Internal(.signalCondition(cond, message, call))
        .Internal(.dfltStop(message, call))
    } else
        .Internal(stop(call., .makeMessage(..., domain = domain)))
}"#,
    ),
    (
        ".makeMessage",
        r#"function(..., domain = NULL, appendLF = FALSE)
 {
    msg <- if(...length()) {
        args <- lapply(list(...), as.character)
        if(is.null(domain) || !is.na(domain))
            args <- .Internal(gettext(domain, unlist(args), TRUE))
        paste(args, collapse = "")
    } else ""
    if(appendLF) paste0(msg, "\n") else msg
}"#,
    ),
    (
        "inherits",
        "function(x, what, which = FALSE) .Internal(inherits(x, what, which))",
    ),
];
