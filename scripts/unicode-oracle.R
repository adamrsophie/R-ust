# Independent R oracle; run with Rscript --vanilla scripts/unicode-oracle.R OUTPUT.
options(warn = 1)
hex <- function(x) paste(sprintf("%02x", as.integer(charToRaw(x))), collapse = "")
encoded <- function(x) if (nzchar(x)) hex(x) else "-"
fingerprint <- function(x) {
    data <- if (is.character(x)) vapply(x, function(s) if (is.na(s)) "NA" else paste0("x", hex(s)), "")
    else if (is.logical(x)) ifelse(is.na(x), "NA", ifelse(x, "T", "F"))
    else if (is.integer(x)) ifelse(is.na(x), "NA", as.character(x))
    else stop("unsupported fixture type")
    paste(typeof(x), length(x), paste(data, collapse = ","), sep = "|")
}
cases <- c(
    "intToUtf8(function() 1)",
    "utf8ToInt(\"\")", "utf8ToInt(\"ASCII\")", "utf8ToInt(\"é水😀\")",
    "utf8ToInt(NA_character_)", "utf8ToInt(c(\"A\",\"B\"))",
    "utf8ToInt(c(NA_character_,\"B\"))", "utf8ToInt(character(0))",
    "utf8ToInt(NULL)", "utf8ToInt(65)", "utf8ToInt(list(\"A\"))",
    "utf8ToInt(matrix(\"A\",1))", "utf8ToInt(factor(\"A\"))", "utf8ToInt()",
    "intToUtf8(integer(0))", "intToUtf8(integer(0),TRUE)", "intToUtf8(NULL)",
    "intToUtf8(c(65,0,66))", "intToUtf8(c(65,0,66),TRUE)",
    "intToUtf8(c(127,128,2047,2048,65535,65536,1114111))",
    "intToUtf8(c(65,NA_integer_,66))", "intToUtf8(c(65,NA_integer_,66),TRUE)",
    "intToUtf8(c(-1,1114112,55296,56320),TRUE)", "intToUtf8(-1)",
    "intToUtf8(1114112)", "intToUtf8(55296)", "intToUtf8(56320)",
    "intToUtf8(c(55357,56832),allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(55296,56320,56319,57343),allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(65,55357,56832,0,66),allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(55357,65),allow_surrogate_pairs=TRUE)",
    "intToUtf8(55357,allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(56320,55296),allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(55357,NA_integer_),allow_surrogate_pairs=TRUE)",
    "intToUtf8(c(55357,56832),TRUE,TRUE)", "intToUtf8(integer(0),TRUE,TRUE)",
    "intToUtf8(c(65.9,66.1))", "intToUtf8(c(TRUE,FALSE,NA),TRUE)",
    "intToUtf8(c(\"65\",\"66\"))", "intToUtf8(c(\"bad\",\"65\"),TRUE)",
    "intToUtf8(c(Inf,NaN,2147483648),TRUE)", "intToUtf8(list(65,66))",
    "intToUtf8(list(1:2))", "intToUtf8(list(NULL))",
    "intToUtf8(factor(c(\"A\",\"B\")),TRUE)", "intToUtf8(matrix(c(65,66),1))",
    "intToUtf8(65,NA)", "intToUtf8(65,logical(0))", "intToUtf8(65,\"bad\")",
    "intToUtf8(65,list(TRUE))", "intToUtf8(65,c(TRUE,FALSE))",
    "intToUtf8(65,\"TRUE\")", "intToUtf8(65,FALSE,NA)",
    "intToUtf8(65,FALSE,logical(0))", "intToUtf8(65,FALSE,c(TRUE,FALSE))",
    "intToUtf8()", "intToUtf8(m=TRUE,x=c(65,66))",
    ".Internal(utf8ToInt(\"A\",\"B\"))", ".Internal(intToUtf8(65))",
    ".Internal(utf8ToInt(ignored=\"A\"))", ".Internal(intToUtf8(a=65,a=FALSE,a=FALSE))",
    "utf8ToInt<-1;utf8ToInt(\"A\")", "intToUtf8<-1;intToUtf8(65)",
    "withVisible(utf8ToInt(invisible(\"A\")))$visible",
    "withVisible(intToUtf8(invisible(65)))$visible",
    "typeof(utf8ToInt)", "typeof(intToUtf8)",
    "intToUtf8(65,TRUE,list(TRUE))", "intToUtf8(65,TRUE,\"TRUE\")"
)
lines <- c(paste("#", R.version.string, R.version$platform, "UNICODE_CONVERSION_V1"))
for (i in seq_along(cases)) {
    warnings <- character(); error <- ""; value <- NULL
    output <- capture.output({
        value <- tryCatch(withCallingHandlers(eval(parse(text=cases[[i]]), new.env()),
            warning=function(w) { warnings <<- c(warnings, conditionMessage(w)); invokeRestart("muffleWarning") }),
            error=function(e) { error <<- conditionMessage(e); NULL })
    })
    kind <- if (nzchar(error)) "error" else typeof(value)
    data <- if (nzchar(error)) "-" else fingerprint(value)
    warning_text <- paste0(if (length(warnings)) paste0("Warning: ", warnings, "\n") else "", collapse="")
    stdout <- if (length(output)) paste0(paste(output,collapse="\n"),"\n") else ""
    lines <- c(lines, paste(i, cases[[i]], kind, data, encoded(error), encoded(warning_text), encoded(stdout), sep="\t"))
}
writeLines(lines, commandArgs(TRUE)[[1]], useBytes=TRUE)
