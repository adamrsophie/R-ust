script <- normalizePath(sub("^--file=", "", grep("^--file=", commandArgs(FALSE), value=TRUE)[1L]))
original <- parse(file=file.path(dirname(script), "run-length-oracle.R"))
eval(original[[1L]])
hex <- function(bytes) paste(format(bytes), collapse="")
cases <- c(
  unused_positional = 'rle(print(1),print(2))',
  unused_dots = 'inverse.rle(list(lengths=1L,values=7L),stop("unused dots"))',
  unused_named = 'rle(x=stop("forced"),bogus=stop("unused"))',
  alias_inverse = 'f<-inverse.rle;inverse.rle<-function(...)stop("masked");f(list(lengths=1L,values=7L),stop("unused dots"))',
  alias_rle = 'f<-rle;rle<-function(...)stop("masked");f(c(1L,1L,2L))$lengths',
  isolated_rle_helpers = 'length<-function(...)stop("masked");is.vector<-length;is.list<-length;rle(c(1L,1L,2L))$lengths',
  isolated_inverse_helper = 'rep.int<-function(...)stop("masked");inverse.rle(list(lengths=2L,values=7L))',
  caller_x = 'x<-c(1L,1L,2L);rle(x)$lengths',
  missing_caller_x = 'rle(x)',
  closure_rle = 'typeof(rle)',
  closure_inverse = 'typeof(inverse.rle)',
  private_rle = '.Internal(rle(stop("forced")))',
  private_inverse = '.Internal(inverse.rle(stop("forced")))',
  force_x_not_dots = 'inverse.rle({print(7L);list(lengths=1L,values=7L)},print(9L))',
  force_once = 'rle({cat("once");c(1L,1L)})$lengths',
  ordered_warnings = 'inverse.rle(list(lengths=list("bad","worse"),values=1:2))'
)
cat("id\tsource\tstatus\tvalue_or_error\tstdout_hex\twarnings_hex\n")
for (id in names(cases)) {
  warnings <- character()
  path <- tempfile()
  sink(path)
  outcome <- tryCatch(withCallingHandlers({
    value <- eval(parse(text=cases[[id]]), envir=new.env(parent=baseenv()))
    c("OK", encode(value))
  }, warning=function(w) {
    warnings <<- c(warnings, conditionMessage(w))
    invokeRestart("muffleWarning")
  }), error=function(e) c("ERR", conditionMessage(e)))
  sink()
  stdout <- readBin(path, "raw", n=file.info(path)$size)
  unlink(path)
  cat(id, cases[[id]], outcome[[1L]], outcome[[2L]], hex(stdout),
      hex(charToRaw(paste(warnings, collapse="\037"))), sep="\t")
  cat("\n")
}
