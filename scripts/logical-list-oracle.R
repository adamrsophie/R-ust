stopifnot(getRversion() == "4.6.0")
compiler::enableJIT(0)
hex <- function(x) if(!nzchar(x)) "-" else paste(sprintf("%02x", as.integer(charToRaw(x))),collapse="")
sources <- c(
  "as.logical(list())", "as.logical(list(TRUE))", "as.logical(list(FALSE,TRUE,NA))",
  "as.logical(list(0L,1L,-1L,NA_integer_))", "as.logical(list(0,NaN,Inf,-Inf))",
  "as.logical(list('TRUE','FALSE','T','F','true','True','false','False'))",
  "as.logical(list('bad','',NA_character_))", "as.logical(list(logical(0)))",
  "as.logical(list(integer(0),numeric(0),character(0)))", "as.logical(list(NULL))",
  "as.logical(list(c(TRUE,FALSE)))", "as.logical(list(list(TRUE)))",
  "as.logical(list(list()))", "as.logical(list(list(TRUE,FALSE)))",
  "as.logical(list(function() 1))", "as.logical(list(factor('x')))",
  "as.logical(list(factor(character(0))))", "as.logical(list(matrix(1L,1)))",
  "as.logical(list(matrix('TRUE',1)))", "as.logical(list(matrix(c(1,2),1)))",
  "as.logical(list(a=TRUE,b=FALSE))", "as.logical(data.frame(a=1,b=0))",
  "as.logical(data.frame(a=c(1,2)))", "as.logical(matrix('TRUE',1))"
)
rows <- character(length(sources))
for(i in seq_along(sources)) {
  error <- ""; warnings <- character()
  value <- tryCatch(withCallingHandlers(eval(parse(text=sources[i])),warning=function(w) {
    warnings <<- c(warnings,conditionMessage(w)); invokeRestart('muffleWarning')
  }),error=function(e) {error <<- conditionMessage(e);NULL})
  kind <- if(nzchar(error)) 'error' else typeof(value)
  payload <- if(nzchar(error)) '-' else paste(ifelse(is.na(value),'NA',ifelse(value,'T','F')),collapse=',')
  rows[i] <- paste(i,sources[i],kind,if(nzchar(payload)) payload else '-',hex(error),hex(paste(warnings,collapse='\n')),sep='\t')
}
writeLines(c(paste('#',R.version.string,R.version$platform,'JIT 0'),rows),'crates/r_ust_eval/tests/fixtures/logical-lists-r-4.6.0.tsv')
cat(length(rows),'logical list oracle rows\n')
