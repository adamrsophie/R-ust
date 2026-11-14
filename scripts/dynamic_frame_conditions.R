base::globalCallingHandlers(
  warning = function(condition) {
    base::cat("R_UST_D1_WARNING=", base::paste(base::sprintf("%02x", base::as.integer(base::charToRaw(base::conditionMessage(condition)))), collapse=""), "\n", sep="", file=base::stderr())
  },
  error = function(condition) {
    base::cat("R_UST_D1_ERROR=", base::paste(base::sprintf("%02x", base::as.integer(base::charToRaw(base::conditionMessage(condition)))), collapse=""), "\n", sep="", file=base::stderr())
  }
)
