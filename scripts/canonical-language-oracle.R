options(warn = 1, keep.source = FALSE)
.wanted <- commandArgs(TRUE)
stopifnot(length(.wanted) <= 1L)
if (identical(.wanted, "--profile")) { cat(R.version.string, R.version$platform, Sys.getlocale(), Sys.getenv("R_ENABLE_JIT"), getOption("keep.source"), sep = "\n"); cat("\n") }
if (length(.wanted) == 0L || identical(.wanted, "symbol")) {
.subject <- quote(expr = x)
cat("M", "symbol", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "operator_symbol")) {
.subject <- quote(expr = `+`)
cat("M", "operator_symbol", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "non_syntactic_symbol")) {
.subject <- quote(expr = `a b`)
cat("M", "non_syntactic_symbol", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "integer")) {
.subject <- quote(expr = 1L)
cat("M", "integer", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "double")) {
.subject <- quote(expr = 1)
cat("M", "double", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "logical")) {
.subject <- quote(expr = TRUE)
cat("M", "logical", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "missing_integer")) {
.subject <- quote(expr = NA_integer_)
cat("M", "missing_integer", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "nan")) {
.subject <- quote(expr = NaN)
cat("M", "nan", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "null")) {
.subject <- quote(expr = NULL)
cat("M", "null", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "string")) {
.subject <- quote(expr = "abc")
cat("M", "string", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "empty_call")) {
.subject <- quote(expr = f())
cat("M", "empty_call", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "tagged_call")) {
.subject <- quote(expr = f(a = x, b = 1L))
cat("M", "tagged_call", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "missing_call")) {
.subject <- quote(expr = f(, y, ))
cat("M", "missing_call", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "missing_call", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "missing_call", "4", typeof(.subject[[4L]]), class(.subject[[4L]]), length(.subject[[4L]]), is.null(.subject[[4L]]), identical(.subject[[4L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "named_missing")) {
.subject <- quote(expr = f(a = , b = y))
cat("M", "named_missing", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "named_missing", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "dots_call")) {
.subject <- quote(expr = f(..., ..1))
cat("M", "dots_call", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "dots_call", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "dots_call", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "binary")) {
.subject <- quote(expr = x + y)
cat("M", "binary", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "prefix_binary")) {
.subject <- quote(expr = `+`(x, y))
cat("M", "prefix_binary", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "unary")) {
.subject <- quote(expr = -x)
cat("M", "unary", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "negative_integer")) {
.subject <- quote(expr = -1L)
cat("M", "negative_integer", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "negative_integer", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "parentheses")) {
.subject <- quote(expr = (x))
cat("M", "parentheses", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "empty_block")) {
.subject <- quote(expr = {})
cat("M", "empty_block", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "block")) {
.subject <- quote(expr = { x <- 1L; x + y })
cat("M", "block", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "if")) {
.subject <- quote(expr = if (x) y)
cat("M", "if", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "if_else")) {
.subject <- quote(expr = if (x) y else z)
cat("M", "if_else", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "while")) {
.subject <- quote(expr = while (x) y)
cat("M", "while", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "for")) {
.subject <- quote(expr = for (x in y) z)
cat("M", "for", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "repeat")) {
.subject <- quote(expr = repeat break)
cat("M", "repeat", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "break")) {
.subject <- quote(expr = break)
cat("M", "break", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "next")) {
.subject <- quote(expr = next)
cat("M", "next", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "return")) {
.subject <- quote(expr = return(x))
cat("M", "return", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "subset_empty")) {
.subject <- quote(expr = x[])
cat("M", "subset_empty", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "subset_empty", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "subset_2d")) {
.subject <- quote(expr = x[, y])
cat("M", "subset_2d", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "subset_2d", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "extract")) {
.subject <- quote(expr = x[[1L]])
cat("M", "extract", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "dollar_symbol")) {
.subject <- quote(expr = x$a)
cat("M", "dollar_symbol", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "dollar_symbol", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "dollar_string")) {
.subject <- quote(expr = x$"a")
cat("M", "dollar_string", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "dollar_string", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "assignment")) {
.subject <- quote(expr = x <- 1L)
cat("M", "assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "right_assignment")) {
.subject <- quote(expr = 1L -> x)
cat("M", "right_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "super_assignment")) {
.subject <- quote(expr = x <<- 1L)
cat("M", "super_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "equals_block")) {
.subject <- quote(expr = { x = 1L })
cat("M", "equals_block", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "equals_block", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "equals_parentheses")) {
.subject <- quote(expr = (x = 1L))
cat("M", "equals_parentheses", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "equals_parentheses", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "subset_assignment")) {
.subject <- quote(expr = x[1L] <- y)
cat("M", "subset_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "subset_assignment", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "extract_assignment")) {
.subject <- quote(expr = x[[1L]] <- y)
cat("M", "extract_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "extract_assignment", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "dollar_assignment")) {
.subject <- quote(expr = x$"a" <- y)
cat("M", "dollar_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "dollar_assignment", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "dollar_assignment", "2/3", typeof(.subject[[2L]][[3L]]), class(.subject[[2L]][[3L]]), length(.subject[[2L]][[3L]]), is.null(.subject[[2L]][[3L]]), identical(.subject[[2L]][[3L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "names_assignment")) {
.subject <- quote(expr = names(x) <- y)
cat("M", "names_assignment", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "names_assignment", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "empty_function")) {
.subject <- quote(expr = function() NULL)
cat("M", "empty_function", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "empty_function", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "empty_function", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "empty_function", "4", typeof(.subject[[4L]]), class(.subject[[4L]]), length(.subject[[4L]]), is.null(.subject[[4L]]), identical(.subject[[4L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "function_defaults")) {
.subject <- quote(expr = function(x, y = NULL, z = 1L, ...) { x; y })
cat("M", "function_defaults", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "function_defaults", "2", typeof(.subject[[2L]]), class(.subject[[2L]]), length(.subject[[2L]]), is.null(.subject[[2L]]), identical(.subject[[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "2/1", typeof(.subject[[2L]][[1L]]), class(.subject[[2L]][[1L]]), length(.subject[[2L]][[1L]]), is.null(.subject[[2L]][[1L]]), identical(.subject[[2L]][[1L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "2/2", typeof(.subject[[2L]][[2L]]), class(.subject[[2L]][[2L]]), length(.subject[[2L]][[2L]]), is.null(.subject[[2L]][[2L]]), identical(.subject[[2L]][[2L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "2/3", typeof(.subject[[2L]][[3L]]), class(.subject[[2L]][[3L]]), length(.subject[[2L]][[3L]]), is.null(.subject[[2L]][[3L]]), identical(.subject[[2L]][[3L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "2/4", typeof(.subject[[2L]][[4L]]), class(.subject[[2L]][[4L]]), length(.subject[[2L]][[4L]]), is.null(.subject[[2L]][[4L]]), identical(.subject[[2L]][[4L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "3", typeof(.subject[[3L]]), class(.subject[[3L]]), length(.subject[[3L]]), is.null(.subject[[3L]]), identical(.subject[[3L]], quote(expr = )), sep = "\t"); cat("\n")
cat("P", "function_defaults", "4", typeof(.subject[[4L]]), class(.subject[[4L]]), length(.subject[[4L]]), is.null(.subject[[4L]]), identical(.subject[[4L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "string_head")) {
.subject <- quote(expr = "sum"(1L))
cat("M", "string_head", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "string_head", "1", typeof(.subject[[1L]]), class(.subject[[1L]]), length(.subject[[1L]]), is.null(.subject[[1L]]), identical(.subject[[1L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "parenthesized_head")) {
.subject <- quote(expr = (f)(x))
cat("M", "parenthesized_head", typeof(.subject), class(.subject), length(.subject), sep = "\t"); cat("\n")
cat("P", "parenthesized_head", "1", typeof(.subject[[1L]]), class(.subject[[1L]]), length(.subject[[1L]]), is.null(.subject[[1L]]), identical(.subject[[1L]], quote(expr = )), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_binary")) {
.left <- quote(expr = x + y)
.right <- quote(expr = `+`(x, y))
cat("E", "eq_binary", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_unary")) {
.left <- quote(expr = -x)
.right <- quote(expr = `-`(x))
cat("E", "eq_unary", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_unary_arity")) {
.left <- quote(expr = -x)
.right <- quote(expr = `-`(x, y))
cat("E", "ne_unary_arity", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_right_assignment")) {
.left <- quote(expr = 1L -> x)
.right <- quote(expr = x <- 1L)
cat("E", "eq_right_assignment", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_right_super")) {
.left <- quote(expr = 1L ->> x)
.right <- quote(expr = x <<- 1L)
cat("E", "eq_right_super", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_assignment_operator")) {
.left <- quote(expr = { x = 1L })
.right <- quote(expr = { x <- 1L })
cat("E", "ne_assignment_operator", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_dollar_member")) {
.left <- quote(expr = x$a)
.right <- quote(expr = x$"a")
cat("E", "ne_dollar_member", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_dollar_escape")) {
.left <- quote(expr = x$"\x61")
.right <- quote(expr = x$"a")
cat("E", "eq_dollar_escape", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_dollar_assignment")) {
.left <- quote(expr = x$a <- y)
.right <- quote(expr = x$"a" <- y)
cat("E", "ne_dollar_assignment", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_tags")) {
.left <- quote(expr = f(a = x))
.right <- quote(expr = f(x))
cat("E", "ne_tags", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_tag_order")) {
.left <- quote(expr = f(a = x, b = y))
.right <- quote(expr = f(b = y, a = x))
cat("E", "ne_tag_order", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_holes")) {
.left <- quote(expr = f(, x, ))
.right <- quote(expr = f(, x, ))
cat("E", "eq_holes", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_hole_null")) {
.left <- quote(expr = f(, x))
.right <- quote(expr = f(NULL, x))
cat("E", "ne_hole_null", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_parentheses")) {
.left <- quote(expr = (x))
.right <- quote(expr = x)
cat("E", "ne_parentheses", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_nested_nan")) {
.left <- quote(expr = f(NaN))
.right <- quote(expr = f(NaN))
cat("E", "eq_nested_nan", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_nan_na")) {
.left <- quote(expr = f(NaN))
.right <- quote(expr = f(NA_real_))
cat("E", "ne_nan_na", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_default_null")) {
.left <- quote(expr = function(x) x)
.right <- quote(expr = function(x = NULL) x)
cat("E", "ne_default_null", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_function")) {
.left <- quote(expr = function(x, y = 1L) x)
.right <- quote(expr = function(x, y = 1L) x)
cat("E", "eq_function", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "ne_formal_order")) {
.left <- quote(expr = function(x, y) x)
.right <- quote(expr = function(y, x) x)
cat("E", "ne_formal_order", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "eq_string_head")) {
.left <- quote(expr = "sum"(1L))
.right <- quote(expr = sum(1L))
cat("E", "eq_string_head", identical(.left, .right), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "runtime_access")) {
.subject <- { list(a = 1L)$"a" }
cat("R", "runtime_access", typeof(.subject), class(.subject), length(.subject), as.character(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "runtime_assignment")) {
.subject <- { x <- list(a = 1L); x$"a" <- 2L; x$a }
cat("R", "runtime_assignment", typeof(.subject), class(.subject), length(.subject), as.character(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "runtime_nonsyntactic")) {
.subject <- { list(`a b` = 3L)$"a b" }
cat("R", "runtime_nonsyntactic", typeof(.subject), class(.subject), length(.subject), as.character(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "runtime_escaped")) {
.subject <- { list(a = 1L)$"\x61" }
cat("R", "runtime_escaped", typeof(.subject), class(.subject), length(.subject), as.character(.subject), sep = "\t"); cat("\n")
}
if (length(.wanted) == 0L || identical(.wanted, "runtime_nonsyntactic_assignment")) {
.subject <- { x <- list(`a b` = 1L); x$"a b" <- 4L; x$`a b` }
cat("R", "runtime_nonsyntactic_assignment", typeof(.subject), class(.subject), length(.subject), as.character(.subject), sep = "\t"); cat("\n")
}
