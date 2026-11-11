args <- commandArgs(TRUE)
snips <- read.delim(args[1], sep = "\t", quote = "", header = TRUE, stringsAsFactors = FALSE, comment.char = "", na.strings = character(0), colClasses = "character")
res <- character(nrow(snips))
for (i in seq_len(nrow(snips))) {
  code <- snips$code[i]
  options(digits = if (endsWith(code, "# default digits")) 7 else 12)
  set.seed(NULL)
  env <- new.env(parent = globalenv())
  out <- tryCatch(capture.output({
    for (e in parse(text = code)) { v <- withVisible(eval(e, env)); if (v$visible) print(v$value) }
  }), error = function(e) paste("ERROR:", conditionMessage(e)))
  out <- gsub("\n", "\\n", out, fixed = TRUE)
  res[i] <- paste(out, collapse = "\\n")
}
writeLines(res, args[2])
