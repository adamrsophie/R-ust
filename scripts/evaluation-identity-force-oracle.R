# R 4.6.0 identity/force oracle. Run: Rscript --vanilla SCRIPT [OUTPUT.tsv]
stopifnot(getRversion() == "4.6.0")
invisible(compiler::enableJIT(0))
options(warn=1)
args <- commandArgs(TRUE)
out <- if(length(args)) args[1] else "crates/r_ust_eval/tests/fixtures/evaluation-identity-force-r-4.6.0.tsv"
hex <- function(x) if(!length(x)) "-" else paste(sprintf("%02x",as.integer(x)),collapse="")
text_hex <- function(x) hex(charToRaw(x))
canonical <- function(x) {
  kind <- typeof(x)
  if(kind == "NULL") return("NULL|0|")
  values <- if(kind == "character") vapply(x,function(y) if(is.na(y)) "NA" else paste0("x",if(nzchar(y)) text_hex(y) else ""),"") else if(kind == "logical") ifelse(is.na(x),"NA",ifelse(x,"T","F")) else if(kind == "integer") ifelse(is.na(x),"NA",as.character(x)) else stop("oracle unsupported canonical type")
  paste0(kind,"|",length(x),"|",paste(values,collapse=","))
}
patterns <- c(
  integer="FUN(3L)", named="FUN(x=7L)", null="FUN(NULL)", empty="FUN(integer())",
  logical="FUN(c(TRUE,FALSE,NA))", string="FUN(c('a','',NA_character_))",
  integers="FUN(c(-1L,NA_integer_,9L))", real_storage="identical(FUN(c(1.5,NA_real_,NaN,Inf)),c(1.5,NA_real_,NaN,Inf))",
  list_value="identical(FUN(list(1L,'a',NULL)),list(1L,'a',NULL))",
  matrix_value="identical(FUN(matrix(1:4,2)),matrix(1:4,2))",
  missing="FUN()", explicit_missing="FUN(x=)", blank="FUN(,)",
  duplicate="FUN(x=stop('FIRST'),x=stop('SECOND'))",
  unused="FUN(1L,extra=stop('UNUSED'))", too_many="FUN(1L,2L)",
  overlong_not_partial="FUN(xx=stop('UNUSED'))",
  exact_before_positional="FUN({cat('BAD');2L},x=1L)",
  supplied_effect="FUN({cat('A');4L})", stop_effect="FUN({cat('A');stop('BOOM');cat('BAD')})",
  unforced="(function(x) 7L)(FUN(stop('BAD')))",
  forced_once="(function(x){FUN(x);FUN(x);x})({cat('ONCE');8L})",
  default="(function(x={cat('DEFAULT');9L}) FUN(x))()",
  default_dependency="(function(x=y,y=8L) FUN(x))()",
  unused_default="(function(x=stop('BAD')) FUN(7L))()",
  dots="(function(...) FUN(...))({cat('DOT');6L})",
  dots_named="(function(...) FUN(...))(x=6L)",
  dots_unused="(function(...) FUN(...))(1L,extra=stop('UNUSED'))",
  dots_duplicate="(function(...) FUN(...))(x=1L,x=2L)",
  alias="{alias<-FUN;alias(8L)}", parameter_alias="(function(f)f(8L))(FUN)",
  shadow="{FUN<-function(x) x+1L;FUN(4L)}",
  shadow_non_function="{FUN<-99L;FUN(4L)}",
  caller_x="{x<-99L;FUN(4L)}",
  caller_binding="{y<-11L;(function(y)FUN(y))(4L)}",
  invisible="FUN(invisible(4L))", assignment_visibility="FUN(z<-4L)",
  nested_visibility="FUN(FUN(invisible(4L)))",
  repeated_visibility="(function(x){FUN(x);FUN(x) })(invisible(4L))",
  return_value="(function(){FUN(return(7L));cat('BAD');9L})()",
  return_invisible="(function(){FUN(return(invisible(7L)));cat('BAD');9L})()",
  return_effect="(function(){FUN({cat('A');return(7L);cat('BAD')});cat('BAD');9L})()",
  top_return="FUN(return(7L))", top_break="FUN(break)", top_next="FUN(next)",
  loop_break="{v<-0L;for(i in 1:3){FUN({cat('A');break});v<-v+1L};v}",
  loop_next="{v<-0L;for(i in 1:3){FUN({cat('A');next});v<-v+1L};v}",
  loop_return="(function(){for(i in 1:3){FUN(return(8L))};99L})()",
  closure_retained="{make<-function(x){FUN(x);function()x};f<-make({cat('CAPTURE');8L});f()}",
  closure_lazy_after="{make<-function(x)FUN(function()x);f<-make({cat('LATER');8L});f()}",
  changed_binding="{y<-3L;f<-(function(x){FUN(x);function()x})(y);y<-9L;f()}",
  closure_storage="typeof(FUN(function(x)x))"
)
scratch <- if(length(args) > 1L) args[2L] else file.path("target","evaluation-identity-force-oracle")
dir.create(scratch,recursive=TRUE,showWarnings=FALSE)
finish <- function() {
  sink()
  output <- rawConnectionValue(.oracle_connection)
  close(.oracle_connection)
  kind <- if(nzchar(.oracle_error)) "error" else typeof(.oracle_value$value)
  value <- if(nzchar(.oracle_error)) text_hex(.oracle_error) else canonical(.oracle_value$value)
  visible <- if(nzchar(.oracle_error)) "-" else if(.oracle_value$visible) "TRUE" else "FALSE"
  writeLines(paste(kind,value,visible,hex(output),text_hex(paste(.oracle_warnings,collapse="\n")),sep="\t"),.oracle_output)
}
functions <- c(paste("hex <-",paste(deparse(hex),collapse="\n")),paste("text_hex <-",paste(deparse(text_hex),collapse="\n")),paste("canonical <-",paste(deparse(canonical),collapse="\n")),paste(".oracle_finish <-",paste(deparse(finish),collapse="\n")))
rows <- character()
for(name in c("identity","force")) {
  f <- get(name,baseenv())
  stopifnot(identical(names(formals(f)),"x"),identical(body(f),quote(x)),identical(environment(f),asNamespace("base")))
  for(id in names(patterns)) {
    source <- gsub("FUN",name,patterns[[id]],fixed=TRUE)
    stem <- file.path(scratch,paste(name,id,sep="-"))
    program <- paste0(stem,".R")
    result <- paste0(stem,".tsv")
    code <- c("stopifnot(getRversion() == '4.6.0')", "invisible(compiler::enableJIT(0))", "options(warn=1)",functions,
      paste0(".oracle_output <- ",encodeString(normalizePath(result,mustWork=FALSE),quote='"')),
      ".oracle_error <- ''; .oracle_warnings <- character()",
      ".oracle_connection <- rawConnection(raw(),'w'); sink(.oracle_connection)",
      "globalCallingHandlers(error=function(e) .oracle_error <<- conditionMessage(e),warning=function(w){.oracle_warnings <<- c(.oracle_warnings,conditionMessage(w));invokeRestart('muffleWarning')})",
      "options(error=function(){.oracle_finish();quit(save='no',status=0,runLast=FALSE)})",
      paste0(".oracle_value <- withVisible({",source,"})"),
      ".oracle_finish()")
    writeLines(code,program)
    status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(normalizePath(program))),stdout=paste0(stem,".stdout"),stderr=paste0(stem,".stderr"))
    stopifnot(status == 0L, file.exists(result))
    data <- readLines(result,warn=FALSE)
    stopifnot(length(data)==1L)
    rows <- c(rows,paste(name,id,source,data,sep="\t"))
  }
}
dir.create(dirname(out),recursive=TRUE,showWarnings=FALSE)
writeLines(c("owner\tid\tsource\ttype\tvalue\tvisible\tstdout_hex\twarnings_hex",rows),out)
cat(length(rows),"independent R 4.6.0 direct-top-level rows\n")
