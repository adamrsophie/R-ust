stopifnot(getRversion() == "4.6.0")
stopifnot(Sys.setlocale("LC_CTYPE", "en_US.UTF-8") == "en_US.UTF-8")
options(warn=1, useFancyQuotes=TRUE)
hex <- function(x) if (!length(charToRaw(x))) "-" else paste(format(charToRaw(x)),collapse="")
cases <- c(
 r"---(encodeString(c("a", "", NA)))---",
 r"---(encodeString(c("a", "", NA), quote="\""))---",
 r"---(encodeString(c("a", "", NA), quote="\"", width=NA))---",
 r"---(encodeString(c("a", "abcd", NA), width=NULL))---",
 r"---(encodeString(c("a", "abcd", NA), width=NA, na.encode=FALSE))---",
 r"---(encodeString(c("a", "abcd", NA), width=8))---",
 r"---(encodeString(c("a", "abcd", NA), width=8, justify="right"))---",
 r"---(encodeString(c("a", "abcd", NA), width=8, justify="centre"))---",
 r"---(encodeString(c("a", "abcd", NA), width=8, justify="none"))---",
 r"---(encodeString(c("a", "abcd", NA), width=8, quote="\"", justify="c"))---",
 r"---(encodeString(c("a", "abcd", NA), width=NA, quote="\"", na.encode=FALSE))---",
 r"---(encodeString(c("a\\b", "\n\t\r\a\b\f\v", "\001\002\037\177")))---",
 r"---(encodeString(c("a\"b", "a'b", "a`b"), quote="\""))---",
 r"---(encodeString(c("a\"b", "a'b", "a`b"), quote="'"))---",
 r"---(encodeString(c("a\"b", "a'b", "a`b"), quote="`"))---",
 r"---(encodeString(c("é", "界", "🙂", "é", ""), width=5, justify="right"))---",
 r"---(encodeString(c("é", "界", "🙂", "é", ""), width=NA, quote="\""))---",
 r"---(encodeString(c("\u0085", "\u200b", "\u0378"), width=10))---",
 r"---(encodeString(NULL))---", r"---(encodeString(character()))---",
 r"---(encodeString(c(1, 2, NA)))---", r"---(encodeString(c(TRUE,FALSE,NA)))---",
 r"---(encodeString(NA, width=NA, quote="\""))---",
 r"---(encodeString(NA, width=NA, quote="\"", na.encode=FALSE))---",
 r"---(encodeString("a", width=2.9))---", r"---(encodeString("a", width=c(4,2)))---",
 r"---(encodeString("a", width="bad"))---", r"---(encodeString("a", width=-1))---",
 r"---(encodeString("a", width=Inf))---", r"---(encodeString("a", width=numeric()))---",
 r"---(encodeString("a", quote="xy"))---", r"---(encodeString("a", quote=NA_character_))---",
 r"---(encodeString("a", quote=1))---", r"---(encodeString("a", quote=character()))---",
 r"---(encodeString("a", quote=c("a","b")))---",
 r"---(encodeString("a", na.encode=NA))---", r"---(encodeString("a", na.encode="TRUE"))---",
 r"---(encodeString("a", na.encode=NULL))---", r"---(encodeString("a", na.encode=c(FALSE,TRUE)))---",
 r"---(encodeString("a", justify=NULL))---", r"---(encodeString("a", justify="r", width=3))---",
 r"---(encodeString("a", justify="n", width=3))---", r"---(encodeString("a", justify=""))---",
 r"---(encodeString("a", justify=NA))---", r"---(encodeString("a", justify=NA_character_))---",
 r"---(encodeString("a", justify=c("left","right")))---",
 r"---(encodeString("a", justify=c("left","right","centre","none")))---",
 r"---(encodeString("a", justify=character()))---",
 r"---(encodeString(wid=3, x="a", just="r", na.e=TRUE, qu="\""))---",
 r"---(encodeString())---", r"---(encodeString("a", bogus=1))---",
 r"---(encodeString(x="a", x="b"))---",
 r"---(encodeString(x={cat("x\n");"a"}, width={cat("width\n");3}, quote={cat("quote\n");""}, na.encode={cat("na\n");TRUE}, justify={cat("justify\n");"left"}))---",
 r"---(encodeString(x={cat("x\n");"a"}, width={cat("width\n");3}, justify="invalid"))---",
 r"---(encodeString(x=stop("x-first"), justify=stop("justify-second")))---",
 r"---(encodeString("a", width=-1, quote={cat("quote\n");""}, na.encode={cat("na\n");TRUE}))---",
 r"---(.Internal(encodeString(c("a",NA), NA, "\"", 1L, TRUE)))---",
 r"---(.Internal(encodeString(1, -1, 1, 9, NA)))---",
 r"---(.Internal(encodeString("a", 1, "", 9, TRUE)))---",
 r"---(.Internal(encodeString("a", 1, "", 0, TRUE, 1)))---"
)
out <- "crates/r_ust_eval/tests/fixtures/encode-string-r-4.6.0.tsv"
cat("# R 4.6.0 en_US.UTF-8 encodeString v1\n",file=out)
for (i in seq_along(cases)) {
 warnings <- character(); error <- ""; output <- character()
 con <- textConnection("output", "w", local=TRUE); sink(con)
 value <- tryCatch(withCallingHandlers(eval(parse(text=cases[i]),new.env()), warning=function(w) {
   warnings <<- c(warnings,conditionMessage(w)); invokeRestart("muffleWarning")
 }), error=function(e) {error <<- conditionMessage(e); NULL})
 sink(); close(con)
 data <- if (nzchar(error)) "-" else paste(vapply(value,function(x) if(is.na(x)) "NA" else hex(x),""),collapse=",")
 if (!nzchar(data)) data <- "EMPTY"
 cat(paste(i,cases[i],data,hex(error),hex(paste(warnings,collapse="\n")),hex(paste0(paste(output,collapse="\n"),if(length(output)) "\n" else "")),sep="\t"),"\n",sep="",file=out,append=TRUE)
}
cat("oracle rows:", length(cases), "\n")
