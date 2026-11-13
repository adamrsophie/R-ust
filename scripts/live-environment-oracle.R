# Run: Rscript --vanilla scripts/live-environment-oracle.R OUTPUT ARTIFACT_DIRECTORY [INPUT_TSV]
args <- commandArgs(TRUE)
stopifnot(length(args) %in% 2:3, as.character(getRversion()) == "4.6.0")
output <- args[[1]]
artifacts <- args[[2]]
input <- if (length(args) == 3) args[[3]] else "crates/r_ust_eval/tests/fixtures/live-environments-r-4.6.0.tsv"
stopifnot(!file.exists(output), !dir.exists(artifacts))
stopifnot(dir.create(artifacts, recursive = TRUE))
fields <- strsplit(readLines(input, warn = FALSE)[-1], "\t", fixed = TRUE)
stopifnot(length(fields) == 20, all(lengths(fields) == 6))
names <- vapply(fields, `[[`, "", 1)
sources <- vapply(fields, `[[`, "", 2)
stopifnot(!anyDuplicated(names), all(grepl("^[a-z][a-z_]*$", names)), all(nzchar(sources)))
rscript <- normalizePath(file.path(R.home("bin"), "Rscript"))
profile <- c(
    R.version.string, R.version$platform, paste("LDOUBLE", .Machine$sizeof.longdouble),
    "R_ENABLE_JIT=0", "R_DEFAULT_PACKAGES=NULL", "LC_ALL=en_US.UTF-8", "LANG=en_US.UTF-8",
    paste("Rscript", rscript), "Child scripts verify R 4.6.0 and explicitly disable JIT at top level."
)
writeLines(profile, file.path(artifacts, "profile.txt"))
lines <- "# R 4.6.0, JIT disabled, withVisible block evaluated directly at top level."
metadata <- "name\tcommand\tstatus\tstdout_bytes\tstderr_bytes"
for (i in seq_along(names)) {
    script <- file.path(artifacts, paste0(names[[i]], ".structured.R"))
    stdout <- file.path(artifacts, paste0(names[[i]], ".stdout"))
    stderr <- file.path(artifacts, paste0(names[[i]], ".stderr"))
    writeLines(c(
        'stopifnot(as.character(getRversion()) == "4.6.0")',
        'invisible(compiler::enableJIT(0))',
        paste0('.live_result<-withVisible({', sources[[i]], '})'),
        'cat("\\n@@RESULT\\t",typeof(.live_result$value),"\\t",paste(.live_result$value,collapse=","),"\\t",.live_result$visible,sep="")'
    ), script, useBytes = TRUE)
    status <- system2(rscript, c("--vanilla", shQuote(script)), stdout = stdout, stderr = stderr,
        env = c("R_ENABLE_JIT=0", "R_DEFAULT_PACKAGES=NULL", "LC_ALL=en_US.UTF-8", "LANG=en_US.UTF-8"))
    stdout_size <- file.info(stdout)$size
    stderr_size <- file.info(stderr)$size
    metadata <- c(metadata, paste(names[[i]], paste(shQuote(rscript), "--vanilla", shQuote(script)),
        status, stdout_size, stderr_size, sep = "\t"))
    writeLines(metadata, file.path(artifacts, "metadata.tsv"))
    stopifnot(status == 0, stderr_size == 0)
    raw_output <- readBin(stdout, "raw", n = stdout_size)
    parts <- strsplit(rawToChar(raw_output), "\n@@RESULT\t", fixed = TRUE)[[1]]
    stopifnot(length(parts) == 2)
    result <- strsplit(parts[[2]], "\t", fixed = TRUE)[[1]]
    stopifnot(length(result) == 3, result[[1]] %in% c("integer", "logical"), result[[3]] == "TRUE")
    effects <- charToRaw(parts[[1]])
    hex <- if (length(effects)) paste(sprintf("%02x", as.integer(effects)), collapse = "") else "-"
    lines <- c(lines, paste(names[[i]], sources[[i]], result[[1]], result[[2]], result[[3]], hex, sep = "\t"))
}
writeLines(lines, output, useBytes = TRUE)
cat("Independent direct-top-level oracle generated", length(names), "rows; raw streams and metadata in", artifacts, "\n")
