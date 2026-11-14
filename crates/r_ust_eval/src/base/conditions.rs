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
    ("geterrmessage", "function() .Internal(geterrmessage())"),
    (
        "tryCatch",
        r#"function(expr, ..., finally) {
    tryCatchList <- function(expr, names, parentenv, handlers) {
	nh <- length(names)
	if (nh > 1L)
	    tryCatchOne(tryCatchList(expr, names[-nh], parentenv,
                                     handlers[-nh]),
			names[nh], parentenv, handlers[[nh]])
	else if (nh == 1L)
	    tryCatchOne(expr, names, parentenv, handlers[[1L]])
	else expr
    }
    tryCatchOne <- function(expr, name, parentenv, handler) {
	doTryCatch <- function(expr, name, parentenv, handler) {
	    .Internal(.addCondHands(name, list(handler), parentenv,
				    environment(), FALSE))
	    expr
	}
	value <- doTryCatch(return(expr), name, parentenv, handler)
	# The return in the call above will exit tryCatchOne unless
	# the handler is invoked; we only get to this point if the handler
	# is invoked.  If we get here then the handler will have been
	# popped off the internal handler stack.
	if (is.null(value[[1L]])) {
	    # a simple error; message is stored internally
	    # and call is in result; this defers all allocs until
	    # after the jump
	    msg <- .Internal(geterrmessage())
	    call <- value[[2L]]
	    cond <- simpleError(msg, call)
	}
        else if (is.character(value[[1L]])) {
            # if the jump for a simple error is intercepted to handle
            # an on.exit() action then the error message is encoded as
            # a character object at that point
	    msg <- value[[1L]]
	    call <- value[[2L]]
	    cond <- simpleError(msg, call)
        }
	else cond <- value[[1L]]
	value[[3L]](cond)
    }
    if (! missing(finally))
        on.exit(finally)
    handlers <- list(...)
    classes <- names(handlers)
    parentenv <- parent.frame()
    if (length(classes) != length(handlers))
        stop("condition handlers must be specified with a condition class")
    tryCatchList(expr, classes, parentenv, handlers)
}"#,
    ),
    (
        "withCallingHandlers",
        r#"function(expr, ...) {
    handlers <- list(...)
    classes <- names(handlers)
    parentenv <- parent.frame()
    if (length(classes) != length(handlers))
        stop("condition handlers must be specified with a condition class")
    .Internal(.addCondHands(classes, handlers, parentenv, NULL, TRUE))
    expr
}"#,
    ),
    (
        "simpleCondition",
        r#"function(message, call = NULL) {
    class <- c("simpleCondition", "condition")
    structure(list(message=as.character(message), call = call), class=class)
}"#,
    ),
    (
        "simpleError",
        r#"function(message, call = NULL) {
    class <- c("simpleError", "error", "condition")
    structure(list(message=as.character(message), call = call), class=class)
}"#,
    ),
    (
        "conditionMessage",
        r#"function(c) UseMethod("conditionMessage")"#,
    ),
    ("conditionCall", r#"function(c) UseMethod("conditionCall")"#),
    ("conditionMessage.condition", r#"function(c) c$message"#),
    ("conditionCall.condition", r#"function(c) c$call"#),
    (
        "signalCondition",
        r#"function(cond) {
    if (! inherits(cond, "condition"))
        cond <- simpleCondition(cond)
    msg <- conditionMessage(cond)
    call <- conditionCall(cond)
    .Internal(.signalCondition(cond, msg, call))
}"#,
    ),
    (
        ".handleSimpleError",
        r#"function(h, msg, call)
    h(simpleError(msg, call))"#,
    ),
];
