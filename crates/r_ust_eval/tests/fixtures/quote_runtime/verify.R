# R_ENABLE_JIT=0 LC_ALL=C R_DEFAULT_PACKAGES=base R --vanilla --slave -f verify.R --args FIXTURE_DIRECTORY NEW_CAPTURE
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
base <- normalizePath(args[[1L]], mustWork = TRUE)
capture <- normalizePath(args[[2L]], mustWork = TRUE)
source(file.path(base, "reference-helpers.R"))
profile()
fixture <- list.files(capture, pattern = "-r-4\\.6\\.0\\.tsv$", full.names = TRUE)
stopifnot(length(fixture) == 1L)
stopifnot(identical(bytes(file.path(base, "reference.tsv")),
                    bytes(fixture)))
rows <- read_cases(file.path(base, "cases.tsv"))
for (i in seq_len(nrow(rows))) {
    directory <- file.path(capture, rows$id[[i]])
    for (mode in c("separate", "merged")) for (stream in c("stdout", "stderr", "status")) {
        stopifnot(identical(bytes(file.path(directory, paste0("run1-", mode, ".", stream))),
                            bytes(file.path(directory, paste0("run2-", mode, ".", stream)))))
    }
    if (rows$expected_status[[i]] == "0") for (stream in c("stdout", "stderr", "status")) {
        stopifnot(identical(bytes(file.path(directory, paste0("typed1.", stream))),
                            bytes(file.path(directory, paste0("typed2.", stream)))))
    }
}
cat(nrow(rows), "fixture rows match; all repeated direct and typed captures agree\n")
