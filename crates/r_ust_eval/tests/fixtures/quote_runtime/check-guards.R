args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
base <- normalizePath(args[[1L]], mustWork = TRUE)
source(file.path(base, "reference-helpers.R"))
output <- args[[2L]]
stopifnot(!file.exists(output), dir.create(output, recursive = TRUE))
output <- normalizePath(output, mustWork = TRUE)
existing <- file.path(output, "existing")
stopifnot(dir.create(existing))
sentinel <- file.path(existing, "sentinel.txt")
writeLines("owned sentinel", sentinel)
before <- bytes(sentinel)
runtime <- file.path(R.home("bin"), "R")
for (kind in c("existing", "jit", "packages", "locale", "version", "platform")) {
    destination <- if (kind == "existing") existing else file.path(output, kind)
    program <- file.path(base, "generate.R")
    if (kind %in% c("version", "platform")) {
        program <- file.path(output, paste0("simulate-", kind, ".R"))
        override <- if (kind == "version") 'getRversion <- function() numeric_version("4.5.0")'
                    else 'R.version$platform <- "unsupported-platform"'
        writeLines(c(override, sprintf('source("%s")', file.path(base, "generate.R"))), program)
    }
    Sys.setenv(R_ENABLE_JIT = if (kind == "jit") "1" else "0",
               R_DEFAULT_PACKAGES = if (kind == "packages") "NULL" else "base",
               LC_ALL = if (kind == "locale") "en_US.UTF-8" else "C")
    command <- c("--vanilla", "--slave", "-f", shQuote(program), "--args",
                 shQuote(file.path(base, "cases.tsv")), shQuote(destination))
    status <- system2(runtime, command, stdout = file.path(output, paste0(kind, ".stdout")),
                      stderr = file.path(output, paste0(kind, ".stderr")))
    writeLines(as.character(status), file.path(output, paste0(kind, ".status")))
    writeLines(paste(c(shQuote(runtime), command), collapse = " "), file.path(output, paste0(kind, ".command")))
    stopifnot(status == 1L)
    if (kind != "existing") stopifnot(!file.exists(destination))
}
stopifnot(identical(before, bytes(sentinel)), identical(list.files(existing), "sentinel.txt"))
cat("Existing output preserved; five incompatible profiles rejected before output creation\n")
