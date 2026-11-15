// Files src/library/base/R/lapply.R and match.fun.R
// match.fun.R: Copyright (C) 1995-2012 The R Core Team
// Part of the R package, https://www.R-project.org
//
// Copyright (C) 1995-2018 The R Core Team
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
        "lapply",
        r###"function (X, FUN, ...)
{
    FUN <- match.fun(FUN)
    ## internal code handles all vector types, including expressions
    ## However, it would be OK to have attributes which is.vector
    ## disallows.
    if(!is.vector(X) || is.object(X)) X <- as.list(X)
    ## Note ... is not passed down.  Rather the internal code
    ## evaluates FUN(X[i], ...) in the frame of this function
    .Internal(lapply(X, FUN))
}"###,
    ),
    (
        "match.fun",
        r###"function (FUN, descend = TRUE)
{
    if ( is.function(FUN) )
        return(FUN)
    if (!(is.character(FUN) && length(FUN) == 1L || is.symbol(FUN))) {
        ## Substitute in parent
        FUN <- eval.parent(substitute(substitute(FUN)))
        if (!is.symbol(FUN))
            stop(gettextf("'%s' is not a function, character or symbol",
                          deparse(FUN)), domain = NA)
    }
    envir <- parent.frame(2)
    if( descend )
        FUN <- get(as.character(FUN), mode = "function", envir = envir)
    else {
        FUN <- get(as.character(FUN), mode = "any", envir = envir)
        if( !is.function(FUN) )
           stop(gettextf("found non-function '%s'", FUN), domain = NA)
    }
    return(FUN)
}"###,
    ),
];
