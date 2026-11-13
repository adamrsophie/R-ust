encode <- function(x) {
  if (is.null(x)) return("null")
  if (is.factor(x)) return(paste0("factor:", encode(unclass(x)), ":", encode(levels(x))))
  if (is.list(x)) return(paste0("list:", paste(vapply(seq_along(x), function(i) {
    name <- if (is.null(names(x))) "" else names(x)[i]
    paste0(name, "=", encode(x[[i]]))
  }, ""), collapse=";")))
  atom <- function(v) {
    if (is.double(v)) {
      if (is.nan(v)) return("NaN")
      if (is.na(v)) return("NA")
      return(paste(format(writeBin(v, raw(), endian="little")), collapse=""))
    }
    if (is.na(v)) return("NA")
    if (is.character(v)) return(encodeString(v, quote='"'))
    as.character(v)
  }
  paste0(typeof(x), ":", paste(vapply(x, atom, ""), collapse=","))
}
cases <- c(
  "rle(c(1L,1L,2L,2L,2L,1L))", "rle(c(TRUE,TRUE,FALSE,NA,NA,TRUE))",
  "rle(c(NA_integer_,NA_integer_,2L,2L))", "rle(c(0,-0,Inf,Inf,-Inf,-Inf))",
  "rle(c(NA_real_,NaN,NaN,NA_real_,1,1))", "rle(c('a','a',NA_character_,NA_character_,'b'))",
  "rle(integer())", "rle(numeric())", "rle(logical())", "rle(character())",
  "rle(7L)", "rle(list())", "rle(list(a=1L))", "rle(list(NULL))",
  "rle(NULL)", "rle(list(1,1))", "rle(factor('a'))", "rle(matrix(1:4,2))",
  "inverse.rle(rle(c(1L,1L,2L)))", "inverse.rle(rle(c(NA_real_,NaN,1)))",
  "inverse.rle(list(lengths=c(2L,0L,3L),values=c('a','b','c')))",
  "inverse.rle(list(values=c(TRUE,NA,FALSE),lengths=c(2,1,0)))",
  "inverse.rle(list(lengths=c(1.9,0.8,-0.8),values=1:3))",
  "inverse.rle(list(lengths=c(TRUE,FALSE),values=c(3,4)))",
  "inverse.rle(list(lengths=c('2','0'),values=c(3L,4L)))",
  "inverse.rle(list(lengths=list(2L,1L),values=list(a=3,b='x')))",
  "inverse.rle(list(lengths=c(0L,2L),values=list(NULL,TRUE)))",
  "inverse.rle(list(lengths=2L,values=factor('a')))",
  "inverse.rle(list(lengths=integer(),values=integer()))",
  "inverse.rle(list(lengths=integer(),values=character()))",
  "inverse.rle(list(lengths=integer(),values=list()))",
  "inverse.rle(list(lengths=character(),values=integer()))",
  "inverse.rle(list(lengths=-1L,values=1))", "inverse.rle(list(lengths=NA_integer_,values=1))",
  "inverse.rle(list(lengths=NaN,values=1))", "inverse.rle(list(lengths=Inf,values=1))",
  "inverse.rle(list(lengths=1e100,values=1))", "inverse.rle(list(lengths=-Inf,values=1))",
  "inverse.rle(list(lengths='NaN',values=1))",
  "inverse.rle(list(lengths=1:2,values=1))", "inverse.rle(list(values=1))",
  "inverse.rle(list(lengths=1))", "inverse.rle(list(lengths=integer(),values=NULL))",
  "inverse.rle(NULL)", "inverse.rle(1)", "inverse.rle(list(lengths='bad',values=1))",
  "inverse.rle(list(lengths=list(1:2),values=1))",
  "rle(x=c(2L,2L))", "inverse.rle(x=list(lengths=2L,values=3L),unused=TRUE)"
)
cat("# ", R.version.string, "; class rle omitted from canonical representation\n", sep="")
for (source in cases) {
  warnings <- character()
  result <- tryCatch(withCallingHandlers(paste0("OK\t", encode(eval(parse(text=source)))),
    warning=function(w) { warnings <<- c(warnings, conditionMessage(w)); invokeRestart("muffleWarning") }),
    error=function(e) paste0("ERR\t", conditionMessage(e)))
  cat(source, result, paste(warnings, collapse="|"), sep="\t")
  cat("\n")
}
