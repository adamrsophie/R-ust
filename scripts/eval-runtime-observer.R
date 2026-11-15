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
        stopifnot(identical(x, quote))
        return('["primitive","quote"]')
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
box <- list(oracle_result$value)
printed <- utils::capture.output(print(box[[1L]], useSource=FALSE))
fields <- c(hex(encode(box,1L)), typeof(box[[1L]]), hex(paste(class(box[[1L]]),collapse=",")),
            mode(box[[1L]]), as.character(length(box[[1L]])), as.character(oracle_result$visible),
            hex(paste(printed,collapse="\n")))
cat("D3_TYPED_V1\n",paste(fields,collapse="\t"),"\n",sep="")
