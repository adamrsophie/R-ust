// Files src/library/base/R/conditions.R and stop.R
// stop.R: Copyright (C) 1995-2025 The R Core Team
// Part of the R package, https://www.R-project.org
//
// Copyright (C) 1995-2022 The R Core Team
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
        "gettext",
        r###"function(..., domain = NULL, trim = TRUE) {
    char <- unlist(lapply(list(...), as.character))
    .Internal(gettext(domain, char, trim))
}"###,
    ),
    (
        "gettextf",
        r###"function(fmt, ..., domain = NULL, trim = TRUE)
    sprintf(gettext(fmt, domain=domain, trim=trim), ...)"###,
    ),
    (
        "warning",
        r###"function(..., call. = TRUE, immediate. = FALSE,
                    noBreaks. = FALSE, domain = NULL)
{
    if (...length() == 1L && inherits(..1, "condition")) {
        cond <- ..1
        if(nargs() > 1L)
            cat(gettext("additional arguments ignored in warning()"),
                "\n", sep = "", file = stderr())
        message <- conditionMessage(cond)
        call <- conditionCall(cond)
        withRestarts({
                .Internal(.signalCondition(cond, message, call))
                .Internal(.dfltWarn(message, call))
            }, muffleWarning = function() NULL) #**** allow simpler form??
        invisible(message)
    } else
        .Internal(warning(call., immediate., noBreaks.,
                          .makeMessage(..., domain = domain)))
}"###,
    ),
    (
        "simpleWarning",
        r###"function(message, call = NULL) {
    class <- c("simpleWarning", "warning", "condition")
    structure(list(message=as.character(message), call = call), class=class)
}"###,
    ),
    (
        ".signalSimpleWarning",
        r###"function(msg, call)
    withRestarts({
           .Internal(.signalCondition(simpleWarning(msg, call), msg, call))
           .Internal(.dfltWarn(msg, call))
        }, muffleWarning = function() NULL)"###,
    ),
    (
        "suppressWarnings",
        r###"function(expr, classes = "warning") {
    withCallingHandlers(expr,
                        warning = function(w)
                            if (inherits(w, classes))
                                tryInvokeRestart("muffleWarning"))
}"###,
    ),
];
