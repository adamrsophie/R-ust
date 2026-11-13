args <- commandArgs(TRUE)
script <- normalizePath(sub("^--file=", "", grep("^--file=", commandArgs(FALSE), value=TRUE)[1L]))
hex <- function(x) if(!nzchar(x)) "-" else paste(sprintf("%02x", as.integer(charToRaw(x))), collapse="")
if(length(args) && args[1] == "--row") {
  stopifnot(getRversion() == "4.6.0")
  compiler::enableJIT(0)
  source <- readLines(args[2], warn=FALSE)[as.integer(args[3])]
  warnings <- character(); error <- ""; value <- NULL
  output <- utils::capture.output(value <- tryCatch(withCallingHandlers(eval(parse(text=source), new.env(parent=globalenv())), warning=function(w) {
    if(getOption("warn") >= 2) return(NULL)
    warnings <<- c(warnings, conditionMessage(w)); invokeRestart("muffleWarning")
  }), error=function(e) { error <<- conditionMessage(e); NULL }))
  kind <- if(nzchar(error)) "error" else typeof(value)
  fingerprint <- if(nzchar(error)) "-" else paste0(kind,"|",length(value),"|",paste(ifelse(is.na(value),"NA",as.character(value)),collapse=","))
  writeLines(paste(args[3],source,kind,fingerprint,hex(error),hex(paste(warnings,collapse="\n")),hex(paste(output,collapse="\n")),sep="\t"),args[4])
} else {
  sources <- c(
    "sequence(c(2,0,3))", "sequence.default(c(2L,1L),from=3L,by=-2L)",
    "sequence(c(2.9,-0.9,1.1),from=2.8,by=-1.9)", "sequence(c(TRUE,FALSE,TRUE))",
    "sequence(c('2','0','3'))", "sequence(list(2,0,3))", "sequence(factor(c('a','b')))",
    "sequence(NULL)", "sequence(integer(0))", "sequence(numeric(0),from=NA,by=NA)",
    "sequence(c(0,0),from=NA,by=NA)", "sequence(1,from=3,by=NA)", "sequence(2,by=NA)",
    "sequence(1,from=NA)", "sequence(-1)", "sequence(c(2,NA))", "sequence(NaN)",
    "sequence(Inf)", "sequence(-Inf)", "sequence(2147483648)", "sequence('bad')",
    "sequence(c(2,3),from=c(10,20),by=c(2,-2))", "sequence(c(1,2,3),from=c(10,20),by=0)",
    "sequence(2,from=c(10,20))", "sequence(2,from=c(10,20),recycle=FALSE)",
    "sequence(2,from=c(10,20),recycle=TRUE)", "sequence(c(1,2),from=c(10,20,30),by=c(1,2),recycle=TRUE)",
    "sequence(1,by=c(1,2,3),recycle=TRUE)", "sequence(1,from=integer(0))",
    "sequence(1,by=integer(0))", "sequence(NA,from=integer(0),recycle=TRUE)",
    "sequence(-1,by=integer(0),recycle=TRUE)", "sequence(integer(0),from=integer(0))",
    "sequence(integer(0),recycle=NA)", "sequence(1,recycle=NA)", "sequence(1,recycle=NULL)",
    "sequence(1,recycle=c(TRUE,FALSE))", "sequence(1,recycle='TRUE')", "sequence(1,recycle='bad')",
    "sequence(1,recycle=list(TRUE))", "sequence(3,from=2147483647,by=1)",
    "sequence(3,from=-2147483647,by=-1)", "sequence(3,from=2147483647,by=2147483647)",
    "sequence(2,from=1e20)", "sequence(1,by=Inf)", "sequence(2,by='bad')",
    "sequence(c(0,1),from=c(NA,4),by=NA)", "sequence(c(1,0),from=c(4,NA),by=NA)",
    "sequence(c(1,-1),from=NA)", "sequence(-1,recycle=NA)", "sequence(-1,from=integer(0))",
    "sequence(2,from=c(NA,10),recycle=FALSE)", "sequence(c(2,0),from=c(10,NA))",
    "sequence(n=2,fr=4,b=-1,rec=FALSE)", "sequence.default(2,3,4,FALSE,ignored=stop('NO'))",
    "sequence(2,ignored=stop('NO'))", "sequence()", "sequence.default()",
    "sequence(2,nvec=3)", "sequence(nvec=2,nvec=3)", "sequence(2,from=3,from=4)",
    "{cat('A');sequence({cat('N');2},{cat('F');3},{cat('B');1},recycle={cat('R');TRUE})}",
    "sequence({cat('N');-1},from={cat('F');NA},by={cat('B');NA},recycle={cat('R');FALSE})",
    "sequence(integer(0),from=stop('FROM'))", "sequence(-1,from=stop('FROM'))",
    "sequence(1,by=stop('BY'),recycle=stop('RECYCLE'))",
    "sequence(1,from='bad',by='bad',recycle=NA)",
    "{sequence(1,from=c(3,4));sequence(1,from=c(5,6))}",
    "{options(warn=2);tryCatch(sequence(1,from=c(3,4)),error=function(e) cat(conditionMessage(e)));options(warn=1);sequence(1,from=c(5,6))}",
    ".Internal(sequence(2L,3L,1L,FALSE))", ".Internal(sequence(2,3L,1L,FALSE))",
    ".Internal(sequence(2L,3,1L,FALSE))", ".Internal(sequence(2L,3L,1,FALSE))",
    ".Internal(sequence(2L,3L,1L))", ".Internal(sequence(2L,3L,1L,FALSE,1))",
    ".Internal(sequence(2L,3L,1L,integer(0)))", ".Internal(sequence(2L,3L,1L,c(1L,2L)))",
    ".Internal(sequence(2L,3L,1L,list(TRUE,FALSE)))", ".Internal(sequence(1L,c(3L,4L),1L,0L))",
    ".Internal(sequence(integer(0),integer(0),integer(0),TRUE))",
    ".Internal(sequence(1L,3L,1L,'true'))", ".Internal(sequence(factor('x'),1L,1L,FALSE))",
    ".Internal(sequence(matrix(2L,1),1L,1L,FALSE))",
    "{sequence<-1;sequence(2)}", "{sequence<-function(...)stop('NO');.Internal(sequence(2L,1L,1L,FALSE))}",
    "{as.integer<-function(...)stop('NO');sequence(2)}", "{sequence.default<-function(...)stop('NO');sequence(2)}"
  )
  sources <- c(utils::head(sources, -1),
    "sequence({cat('N');2},from=3,from=4)",
    "sequence.default({cat('N');2},from=3,from=4)",
    "sequence(-1,from=c(2,3))",
    "sequence(integer(0),from='bad',by='bad')")
  dir <- file.path("target", "sequence-oracle")
  dir.create(dir, recursive=TRUE, showWarnings=FALSE)
  writeLines(sources,file.path(dir,"cases.R.txt"))
  rows <- unlist(parallel::mclapply(seq_along(sources), function(i) {
    path <- file.path(dir,paste0("oracle-",i,".tsv"))
    status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(script),"--row",shQuote(file.path(dir,"cases.R.txt")),i,shQuote(path)),stdout=file.path(dir,paste0("oracle-launch-",i,".log")),stderr=file.path(dir,paste0("oracle-launch-error-",i,".log")))
    stopifnot(status == 0L)
    readLines(path)
  }, mc.cores=4L))
  header <- paste("#",R.version.string,R.version$platform,"JIT 0 FRESH_PROCESS_PER_ROW")
  writeLines(c(header,rows),"crates/r_ust_eval/tests/fixtures/sequence-r-4.6.0.tsv")
  cat(length(rows),"oracle rows\n")
}
