# Typed observer for the independent environment reference command. No native imports.
local({
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
primitive_names <- c("globalenv", "baseenv", "emptyenv", "is.environment", ".Primitive", "quote", "sum", "length", "invisible", "return", "missing", "+", "-", "*", "/", "^", "%%", "%/%", ":", "if", "sqrt", "[", ".Internal")
hex <- function(x) paste(sprintf("%02x", as.integer(charToRaw(x))), collapse = "")
unhex <- function(x) {
    if (!nzchar(x)) return("")
    rawToChar(as.raw(strtoi(substring(x, seq.int(1L, nchar(x), 2L),
                                    seq.int(2L, nchar(x), 2L)), 16L)))
}
quoted <- function(x) paste0('"', x, '"')
# The canonical node transport matches the independent language printer oracle.
encode <- function(box, index) {
    if (identical(box[[index]], quote(expr = ))) return('["missing"]')
    x <- box[[index]]
    if (is.primitive(x)) {
        names <- primitive_names[vapply(primitive_names, function(name) identical(x, .Primitive(name)), TRUE)]
        stopifnot(length(names) == 1L)
        return(paste0('["primitive",', quoted(names), ']'))
    }
    if (is.environment(x)) return(paste0('["environment",', quoted(environment_role(x)), ']'))
    if (is.list(x) && !is.null(names(x))) {
        stopifnot(identical(names(attributes(x)), "names"))
        children <- vapply(seq_along(x), function(j) paste0('[', quoted(hex(names(x)[[j]])), ',', encode(x, j), ']'), "")
        return(paste0('["named-list",[', paste(children, collapse=","), ']]'))
    }
    stopifnot(is.null(attributes(x)) ||
              (is.pairlist(x) && identical(names(attributes(x)), "names")))
    type <- typeof(x)
    if (is.null(x)) return('["nil"]')
    if (is.symbol(x)) return(paste0('["symbol",', quoted(hex(as.character(x))), ']'))
    if (is.call(x) || is.pairlist(x)) {
        xs <- as.list(x)
        tags <- names(xs)
        first <- if (is.call(x)) 2L else 1L
        rows <- character()
        if (length(xs) >= first) for (j in seq.int(first, length(xs))) {
            tag <- if (is.null(tags) || tags[[j]] == "") "null" else quoted(hex(tags[[j]]))
            rows <- c(rows, paste0('[', tag, ',', encode(xs, j), ']'))
        }
        fields <- paste(rows, collapse = ',')
        if (is.call(x)) return(paste0('["call",', encode(xs, 1L), ',[', fields, ']]'))
        return(paste0('["pairlist",[', fields, ']]'))
    }
    if (type %in% c("list", "expression") || length(x) != 1L) {
        children <- vapply(seq_along(x), function(j) encode(as.list(x), j), "")
        return(paste0('["vector","', type, '",[', paste(children, collapse = ','), ']]'))
    }
    stopifnot(length(x) == 1L, type %in% c("integer", "double", "logical", "character"))
    if (type == "double") {
        bits <- paste(sprintf("%02x", as.integer(writeBin(x, raw(), size = 8L, endian = "big"))), collapse = "")
        return(paste0('["real",', quoted(bits), ']'))
    }
    if (type == "character") value <- if (is.na(x)) "null" else quoted(hex(x))
    else value <- quoted(as.character(x))
    encoding <- if (type == "character") paste0(',', quoted(Encoding(x))) else ""
    paste0('[', quoted(if (type == "character") "string" else type), ',', value, encoding, ']')
}
profile <- function() {
    stopifnot(as.character(getRversion()) == "4.6.0", .Machine$sizeof.longdouble == 8L,
              .Machine$double.digits == 53L, Sys.getlocale() == "C", Sys.getenv("R_ENABLE_JIT") == "0")
    options(warn = 1L, warnPartialMatchArgs = FALSE, digits = 7L, scipen = 0L,
            width = 80L, deparse.cutoff = 60L, keep.source = FALSE)
}

environment_role <- function(x) {
    if (identical(x, globalenv())) return("global")
    if (identical(x, baseenv())) return("base")
    if (identical(x, emptyenv())) return("empty")
    if (identical(x, .BaseNamespaceEnv)) return("base_namespace")
    if (identical(x, .AutoloadEnv)) return("autoloads")
    "ordinary"
}

profile()
rows <- utils::read.delim(args[[1L]], colClasses = "character", check.names = FALSE,
                          quote = "", comment.char = "", na.strings = NULL)
row <- rows[rows$id == args[[2L]], ]
stopifnot(nrow(row) == 1L)
failure <- NULL
environment <- globalenv()
tryCatch(invisible(eval(parse(text = unhex(row$setup_hex), keep.source = FALSE), environment)),
         error = function(condition) { failure <<- conditionMessage(condition); NULL })
warnings <- character()
if (is.null(failure)) {
effects <- utils::capture.output({
    observed <- tryCatch(withCallingHandlers(
        withVisible(eval(parse(text = unhex(row$subject_hex), keep.source = FALSE), environment)),
        warning = function(condition) {
            warnings <<- c(warnings, paste(class(condition), collapse = ","),
                          conditionMessage(condition), paste(deparse(conditionCall(condition)), collapse = "\n"))
            invokeRestart("muffleWarning")
        }), error = function(condition) { failure <<- conditionMessage(condition); NULL })
    invisible(eval(parse(text = unhex(row$after_hex), keep.source = FALSE), environment))
})
}
if (is.null(failure)) {
box <- list(observed$value)
printed <- utils::capture.output(print(box[[1L]], useSource = FALSE))
fields <- c(row$id, row$category, "value", hex(encode(box, 1L)),
            typeof(box[[1L]]), hex(paste(class(box[[1L]]), collapse = ",")),
            mode(box[[1L]]), as.character(length(box[[1L]])),
            as.character(observed$visible), hex(paste(printed, collapse = "\n")),
            hex(paste(warnings, collapse = "\n")), hex(paste(effects, collapse = "\n")))
} else {
    fields <- c(row$id, row$category, "error", rep("", 9L))
}
cat(paste(c(fields, hex(if (is.null(failure)) "" else failure)), collapse = "\t"), "\n", sep = "")
})
