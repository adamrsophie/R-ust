primitive_names <- c(".Primitive", "quote", "sum", "length", "invisible", "return", "missing", "+", "-", "*", "/", "^", "%%", "%/%", ":", "if", "sqrt", "[", ".Internal")
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
read_cases <- function(path) {
    rows <- utils::read.delim(path, colClasses = "character", check.names = FALSE,
                             quote = "", comment.char = "", na.strings = NULL)
    stopifnot(identical(names(rows), c("id", "category", "setup_hex", "subject_hex", "after_hex", "expected_status")))
    stopifnot(nrow(rows) > 0L, !anyDuplicated(rows$id))
    stopifnot(all(grepl("^[a-z][a-z0-9_]*$", rows$id)), all(rows$expected_status %in% c("0", "1")))
    stopifnot(all(grepl("^([0-9a-f]{2})*$", unlist(rows[c("setup_hex", "subject_hex", "after_hex")]))))
    stopifnot(all(nzchar(rows$subject_hex)))
    rows
}
bytes <- function(path) readBin(path, "raw", n = file.info(path)$size)
bytehex <- function(path) paste(sprintf("%02x", as.integer(bytes(path))), collapse = "")
profile <- function() {
    stopifnot(as.character(getRversion()) == "4.6.0", .Machine$sizeof.longdouble == 8L,
              R.version$platform == "aarch64-apple-darwin23",
              .Machine$double.digits == 53L, Sys.getlocale() == "C",
              Sys.getenv("R_DEFAULT_PACKAGES") == "base", Sys.getenv("R_ENABLE_JIT") == "0")
    options(warn = 1L, warnPartialMatchArgs = FALSE, digits = 7L, scipen = 0L,
            width = 80L, deparse.cutoff = 60L, keep.source = FALSE)
}
