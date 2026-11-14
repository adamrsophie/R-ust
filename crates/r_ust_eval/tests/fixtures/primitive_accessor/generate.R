# R_ENABLE_JIT=0 LC_ALL=C R --vanilla --slave -f generate.R --args cases.tsv NEW_OUTPUT_DIRECTORY
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
input <- normalizePath(args[[1L]], mustWork = TRUE)
base <- dirname(input)
source(file.path(base, "reference-helpers.R"))
profile()
if (file.exists(args[[2L]])) stop("output already exists")
rows <- read_cases(input)
stopifnot(dir.create(args[[2L]], recursive = TRUE))
output <- normalizePath(args[[2L]], mustWork = TRUE)
runtime <- file.path(R.home("bin"), "R")
profile_lines <- c(R.version.string, R.version$platform,
                   paste("longdouble", .Machine$sizeof.longdouble),
                   paste("double.digits", .Machine$double.digits),
                   paste("locale", Sys.getlocale()), paste("JIT", Sys.getenv("R_ENABLE_JIT")),
                   paste("R_HOME", R.home()))
writeLines(profile_lines, file.path(output, "profile.txt"))
prelude <- 'options(warn=1L, warnPartialMatchArgs=FALSE, keep.source=FALSE, digits=7L, scipen=0L, width=80L, deparse.cutoff=60L)'
header <- paste(c("id", "category", "outcome", "node_hex", "typeof", "class_hex", "mode", "length",
                  "visible", "print_hex", "warnings_hex", "effects_hex", "status", "stdout_hex", "stderr_hex", "merged_hex"), collapse = "\t")
fixture <- header
execute <- function(command, prefix, merged = FALSE) {
    out <- paste0(prefix, ".stdout")
    err <- paste0(prefix, ".stderr")
    status <- system2(runtime, command, stdout = out, stderr = if (merged) out else err)
    if (merged) file.create(err)
    writeLines(as.character(status), paste0(prefix, ".status"))
    writeLines(paste(c(shQuote(runtime), command), collapse = " "), paste0(prefix, ".command"))
    status
}
failures <- character()
for (i in seq_len(nrow(rows))) {
    row <- rows[i, ]
    directory <- file.path(output, row$id)
    stopifnot(dir.create(directory))
    program <- file.path(directory, "source.R")
    writeLines(c(prelude, unhex(row$setup_hex), unhex(row$subject_hex),
                 unhex(row$after_hex), 'cat("AFTER_SUBJECT_SENTINEL\\n")'), program)
    command <- c("--vanilla", "--slave", "-f", shQuote(program))
    for (iteration in 1:2) for (mode in c("separate", "merged")) {
        prefix <- file.path(directory, paste0("run", iteration, "-", mode))
        status <- execute(command, prefix, mode == "merged")
        if (status != as.integer(row$expected_status)) failures <- c(failures, paste(row$id, iteration, mode, status))
    }
    fields <- c(row$id, row$category, "error", rep("", 9L))
    if (row$expected_status == "0") {
        typed_command <- c("--vanilla", "--slave", "-f", shQuote(file.path(base, "observe.R")),
                           "--args", shQuote(base), shQuote(input), row$id)
        for (iteration in 1:2) {
            status <- execute(typed_command, file.path(directory, paste0("typed", iteration)))
            if (status != 0L) failures <- c(failures, paste(row$id, "typed", iteration, status))
        }
        line <- readLines(file.path(directory, "typed1.stdout"))
        if (length(line) == 1L) fields <- strsplit(paste0(line, "\tEND"), "\t", fixed = TRUE)[[1L]][1:12]
    }
    direct <- file.path(directory, "run1-separate")
    fields <- c(fields, readLines(paste0(direct, ".status")), bytehex(paste0(direct, ".stdout")),
                bytehex(paste0(direct, ".stderr")), bytehex(file.path(directory, "run1-merged.stdout")))
    fixture <- c(fixture, paste(fields, collapse = "\t"))
}
writeLines(fixture, file.path(output, "primitive-accessor-r-4.6.0.tsv"))
writeLines(failures, file.path(output, "failures.txt"))
stopifnot(length(failures) == 0L)
cat(nrow(rows), "subjects captured with two separate/merged direct runs each\n")
