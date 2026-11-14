args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) stop("usage: Rscript operator-primitives-fixture.R CASES.tsv NEW_OUTPUT.tsv")
if (file.exists(args[[2L]])) stop("refusing existing output")
if (getRversion() != "4.6.0" || .Machine$sizeof.longdouble != 8L) stop("requires R 4.6.0 with 8-byte long double")
if (Sys.getenv("LC_ALL") != "C" || Sys.getenv("R_ENABLE_JIT") != "0") stop("requires LC_ALL=C R_ENABLE_JIT=0")
options(warn = 0)
Sys.setenv("_R_CHECK_LENGTH_COLON_" = "false")

hex <- function(bytes) {
    if (!length(bytes)) return("-")
    paste(sprintf("%02x", as.integer(bytes)), collapse = "")
}

capture_bytes <- function(code) {
    path <- tempfile()
    connection <- file(path, "wb")
    depth <- sink.number()
    on.exit({
        while (sink.number() > depth) sink()
        close(connection)
        unlink(path)
    })
    sink(connection)
    value <- force(code)
    sink()
    flush(connection)
    bytes <- readBin(path, "raw", n = file.info(path)$size)
    list(value = value, bytes = bytes)
}

fingerprint <- function(value) {
    storage <- typeof(value)
    if (storage == "builtin") {
        return(paste0("builtin:", deparse(value)))
    }
    tokens <- switch(storage,
        integer = ifelse(is.na(value), "NA", as.character(value)),
        logical = ifelse(is.na(value), "NA", ifelse(value, "T", "F")),
        double = vapply(value, function(number) hex(writeBin(number, raw(), size = 8L, endian = "big")), ""),
        stop(paste("unsupported storage", storage))
    )
    paste(storage, length(value), paste(tokens, collapse = ","), sep = "|")
}

observe <- function(fields) {
    environment <- new.env(parent = baseenv())
    warnings <- character()
    result <- capture_bytes(tryCatch(
        withCallingHandlers(
            withVisible(eval(parse(text = fields[[3L]], keep.source = FALSE), envir = environment)),
            warning = function(condition) {
                warnings <<- c(warnings, conditionMessage(condition))
                invokeRestart("muffleWarning")
            }
        ),
        error = function(condition) condition
    ))
    warning_hex <- hex(charToRaw(paste(warnings, collapse = "\n")))
    if (inherits(result$value, "error")) {
        values <- c("error", "-", hex(charToRaw(conditionMessage(result$value))), warning_hex,
            hex(result$bytes), "-", "-")
    } else {
        value <- result$value$value
        display <- if (typeof(value) == "builtin") hex(capture_bytes(print(value))$bytes) else "-"
        values <- c(typeof(value), fingerprint(value), "-", warning_hex, hex(result$bytes),
            if (result$value$visible) "TRUE" else "FALSE", display)
    }
    paste(c(fields, values), collapse = "\t")
}

lines <- readLines(args[[1L]], warn = FALSE)
stopifnot(identical(lines[[1L]], "id\tscope\tsource"), length(lines) == 279L)
cases <- strsplit(lines[-1L], "\t", fixed = TRUE)
stopifnot(all(lengths(cases) == 3L), !anyDuplicated(vapply(cases, `[[`, "", 1L)))
scopes <- vapply(cases, `[[`, "", 2L)
stopifnot(sum(scopes == "portable") == 246L, sum(scopes == "future_quote") == 32L)
rows <- vapply(cases, observe, "")
header <- "id\tscope\tsource\tstorage\tfingerprint\terror_hex\twarnings_hex\tstdout_hex\tvisible\thandle_display_hex"
writeLines(c(header, rows), args[[2L]], useBytes = TRUE)
