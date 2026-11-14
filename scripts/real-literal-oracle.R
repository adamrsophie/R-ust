# R --vanilla --slave -f scripts/real-literal-oracle.R --args crates/r_ust_eval/tests/fixtures/real-literal-inputs.tsv full-prints.bin
# This observes constructed scalar data, never evaluates a supplied user program.
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 2L)
if (file.exists(args[[2L]])) stop("Refusing to replace existing output")
stopifnot(as.character(getRversion()) == "4.6.0")
stopifnot(.Machine$sizeof.longdouble == 8L)
stopifnot(Sys.setlocale("LC_ALL", "C") != "")
options(digits = 7L, scipen = 0L, width = 80L, deparse.cutoff = 60L, keep.source = FALSE)
stopifnot(base::.deparseOpts(NULL) == 0L)
inputs <- read.delim(args[[1L]], colClasses = "character", check.names = FALSE)
stopifnot(identical(names(inputs), c("id", "bits")))
stopifnot(!anyDuplicated(inputs$id), !anyDuplicated(inputs$bits))
stopifnot(all(grepl("^[0-9a-f]{16}$", inputs$bits)))
hex <- function(text) paste(sprintf("%02x", as.integer(charToRaw(text))), collapse = "")
message(R.version.string)
message("platform=", R.version$platform)
message("sizeof.longdouble=", .Machine$sizeof.longdouble)
message("locale=", Sys.getlocale())
message("digits=", getOption("digits"), ";scipen=", getOption("scipen"),
        ";width=", getOption("width"), ";cutoff=", getOption("deparse.cutoff"))
message("simple.control.flags=", base::.deparseOpts(NULL))
message("cases=", nrow(inputs))
fullprints <- file(args[[2L]], open = "wt")
cat("id\tbits\ttoken_hex\tsimpletoken_hex\tfullprint_hex\n")
for (i in seq_len(nrow(inputs))) {
    bits <- inputs$bits[[i]]
    octets <- substring(bits, seq.int(1L, 15L, 2L), seq.int(2L, 16L, 2L))
    value <- readBin(as.raw(strtoi(octets, base = 16L)), "double", n = 1L,
                     size = 8L, endian = "big")
    # readBin/writeBin must preserve NaN payloads, signs, and every finite input.
    observed <- paste(sprintf("%02x", as.integer(writeBin(value, raw(), size = 8L, endian = "big"))),
                      collapse = "")
    stopifnot(identical(observed, bits), is.null(attributes(value)))
    language <- call("f", value)
    line <- capture.output(print(language))
    stopifnot(length(line) == 1L, startsWith(line, "f("), endsWith(line, ")"), nchar(line, "bytes") < 60L)
    token <- substr(line, 3L, nchar(line) - 1L)
    simple <- deparse(value, width.cutoff = 60L, backtick = TRUE, control = NULL)
    stopifnot(length(simple) == 1L)
    sink(fullprints)
    print(language)
    sink()
    cat(inputs$id[[i]], bits, hex(token), hex(simple), hex(paste0(line, "\n")), sep = "\t")
    cat("\n")
}
close(fullprints)
