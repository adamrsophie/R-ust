// New-Internal.R: Copyright (C) 1995-2023 The R Core Team
// get.R: Copyright (C) 1995-2025 The R Core Team
// backquote.R: Copyright (C) 1995-2017 The R Core Team
// as.R: Copyright (C) 1995-2021 The R Core Team
// Files from src/library/base/R/
// Part of the R package, https://www.R-project.org
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// A copy of the GNU General Public License is available at
// https://www.R-project.org/Licenses/

pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        "do.call",
        r###"function(what, args, quote = FALSE, envir = parent.frame())
{
    if (!is.list(args))
	stop("second argument must be a list")
    if (quote)
	args <- lapply(args, enquote)
    .Internal(do.call(what, args, envir))
}"###,
    ),
    (
        "get",
        r###"function (x, pos = -1L, envir = as.environment(pos), mode = "any",
              inherits = TRUE)
    .Internal(get(x, envir, mode, inherits))"###,
    ),
    (
        "enquote",
        r###"function(cl) as.call(list(quote(base::quote), cl))"###,
    ),
    (
        "as.symbol",
        r###"function(x) .Internal(as.vector(x, "symbol"))"###,
    ),
    (
        "as.name",
        r###"function(x) .Internal(as.vector(x, "symbol"))"###,
    ),
];
