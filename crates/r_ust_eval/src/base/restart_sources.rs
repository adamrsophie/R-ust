// File src/library/base/R/conditions.R
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
    ("isRestart", r###"function(x) inherits(x, "restart")"###),
    (
        "findRestart",
        r###"function(name, cond = NULL) {
    i <- 1L
    repeat {
        r <- .Internal(.getRestart(i))
        if (is.null(r))
            return(NULL)
        else if (name == r[[1L]] &&
                 (is.null(cond) || is.null(r$test) || r$test(cond)))
            return(r)
        else i <- i + 1L
    }
}"###,
    ),
    (
        "computeRestarts",
        r###"function(cond = NULL) {
    val <- NULL
    i <- 1L
    repeat {
        r <- .Internal(.getRestart(i))
        if (is.null(r))
            return(val)
        else if (is.null(cond) || is.null(r$test) || r$test(cond))
            val <- c(val, list(r))
        i <- i + 1L
    }
}"###,
    ),
    (
        "invokeRestart",
        r###"function(r, ...) {
    if (! isRestart(r)) {
        res <- findRestart(r)
        if (is.null(res))
            stop(gettextf("no 'restart' '%s' found", as.character(r)),
                 domain = NA)
        r <- res
    }
    .Internal(.invokeRestart(r, list(...)))
}"###,
    ),
    (
        "tryInvokeRestart",
        r###"function(r, ...) {
    if (!isRestart(r))
        r <- findRestart(r)

    if (is.null(r))
        invisible(NULL)
    else
        .Internal(.invokeRestart(r, list(...)))
}"###,
    ),
    (
        "withRestarts",
        r###"function(expr, ...) {
    docall <- function(fun, args) {
	if ((is.character(fun) && length(fun) == 1L) || is.name(fun))
	    fun <- get(as.character(fun), envir = parent.frame(),
                       mode = "function")
	do.call("fun", lapply(args, enquote))
    }
    makeRestart <- function(name = "",
			   handler = function(...) NULL,
			   description = "",
			   test = function(c) TRUE,
			   interactive = NULL) {
	structure(list(name = name, exit = NULL, handler = handler,
		       description = description, test = test,
		       interactive = interactive),
		  class = "restart")
    }
    makeRestartList <- function(...) {
        specs <- list(...)
        names <- names(specs)
        restarts <- vector("list", length(specs))
        for (i in seq_along(specs)) {
            spec <- specs[[i]]
            name <- names[i]
            if (is.function(spec))
                restarts[[i]] <- makeRestart(handler = spec)
            else if (is.character(spec))
                restarts[[i]] <- makeRestart(description = spec)
            else if (is.list(spec))
                restarts[[i]] <- docall("makeRestart", spec)
            else
               stop("not a valid restart specification")
            restarts[[i]]$name <- name
        }
        restarts
    }
    withOneRestart <- function(expr, restart) {
	doWithOneRestart <- function(expr, restart) {
	    restart$exit <- environment()
	    .Internal(.addRestart(restart))
	    expr
	}
	restartArgs <- doWithOneRestart(return(expr), restart)
	# The return in the call above will exit withOneRestart unless
	# the restart is invoked; we only get to this point if the restart
	# is invoked.  If we get here then the restart will have been
	# popped off the internal restart stack.
	docall(restart$handler, restartArgs)
    }
    withRestartList <- function(expr, restarts) {
	nr <- length(restarts)
	if (nr > 1L)
	    withOneRestart(withRestartList(expr, restarts[-nr]),
                           restarts[[nr]])
	else if (nr == 1L)
	    withOneRestart(expr, restarts[[1L]])
	else expr
    }
    restarts <- makeRestartList(...)
    if (length(restarts) == 0L)
        expr
    else if (length(restarts) == 1L)
        withOneRestart(expr, restarts[[1L]])
    else withRestartList(expr, restarts)
}"###,
    ),
];
