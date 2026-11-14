# R --vanilla --slave -f generate.R --args cases.tsv prints.bin
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L, as.character(getRversion()) == "4.6.0")
if (file.exists(args[[2L]])) stop("output already exists")
stopifnot(.Machine$sizeof.longdouble == 8L, .Machine$double.digits == 53L)
stopifnot(Sys.setlocale("LC_ALL", "C") != "", Sys.getenv("R_ENABLE_JIT") == "0")
options(warn = 1L, digits = 7L, scipen = 0L, width = 80L,
        deparse.cutoff = 60L, keep.source = FALSE)
controls <- c("keepInteger", "keepNA", "niceNames")
stopifnot(base::.deparseOpts(controls) == 1089L)
hex <- function(x) paste(sprintf("%02x", as.integer(charToRaw(x))), collapse = "")
unhex <- function(x) rawToChar(as.raw(strtoi(substring(x, seq.int(1L, nchar(x), 2L),
                                                      seq.int(2L, nchar(x), 2L)), 16L)))
quoted <- function(x) paste0('"', x, '"')
encode <- function(box, index) {
    if (identical(box[[index]], quote(expr = ))) return('["missing"]')
    x <- box[[index]]
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
inputs <- utils::read.delim(args[[1L]], colClasses = "character", check.names = FALSE,
                     quote = "", comment.char = "")
stopifnot(identical(names(inputs), c("id", "category", "constructor_hex")))
stopifnot(nrow(inputs) > 0L, !anyDuplicated(inputs$id))
stopifnot(all(grepl("^[a-z][a-z0-9_]*$", inputs$id)))
stopifnot(all(grepl("^([0-9a-f]{2})+$", inputs$constructor_hex)))
message(R.version.string, ";platform=", R.version$platform,
        ";longdouble=", .Machine$sizeof.longdouble, ";binarydigits=", .Machine$double.digits)
message("locale=", Sys.getlocale(), ";JIT=", Sys.getenv("R_ENABLE_JIT"), ";flags=1089;cutoff=60")
message("MBCS=", l10n_info()$MBCS, ";UTF8=", l10n_info()[["UTF-8"]])
stream <- file(args[[2L]], "wb")
cat("id\tcategory\tnode_hex\toutput_hex\n")
for (i in seq_len(nrow(inputs))) {
    source <- unhex(inputs$constructor_hex[[i]])
    expressions <- parse(text = source, keep.source = FALSE)
    stopifnot(length(expressions) == 1L)
    box <- list(eval(expressions[[1L]], envir = new.env(parent = baseenv())))
    node <- encode(box, 1L)
    lines <- deparse(box[[1L]], width.cutoff = 60L, backtick = TRUE, control = controls)
    output <- paste(lines, collapse = "\n")
    if (is.language(box[[1L]])) {
        observed <- utils::capture.output(print(box[[1L]], useSource = FALSE))
        stopifnot(identical(lines, observed))
    }
    writeBin(charToRaw(paste0(output, "\n")), stream)
    cat(inputs$id[[i]], inputs$category[[i]], hex(node), hex(output), sep = "\t")
    cat("\n")
}
close(stream)
