stopifnot(getRversion() == "4.6.0")
Sys.setenv(R_ENABLE_JIT = "0")
args <- commandArgs(TRUE)
destination <- if(length(args)) args[1] else ".omo/evidence/primitive-values/oracle"
dir.create(destination, recursive=TRUE, showWarnings=FALSE)
sources <- c(
  "sum", "length", "invisible", "`return`", "missing",
  "typeof(sum)", "typeof(length)", "typeof(invisible)", "typeof(`return`)", "typeof(missing)",
  "class(sum)", "class(`return`)", "is.function(sum)", "is.function(missing)",
  "length(sum)", "length(`return`)", "identical(sum,sum)", "identical(sum,length)",
  "identical(`return`,missing)", "{s<-sum;identical(s,sum)}",
  "{s<-sum;s(1L,2L,3L)}", "{s<-sum;s(1,NA,na.rm=TRUE)}",
  "{l<-length;l(1:4)}", "{i<-invisible;i(7L)}", "{i<-invisible;i()}",
  "{r<-`return`;f<-function(){r(9L);99L};f()}",
  "{m<-missing;f<-function(x)m(x);f()}", "{m<-missing;f<-function(x)m(x);f(2)}",
  "{m<-missing;f<-function(x=stop('FORCED'))m(x);f()}",
  "{f<-function(fun)fun(2L,3L);f(sum)}",
  "{s<-sum;sum<-function(...)99L;s(2L,3L)}", "{sum<-function(...)99L;sum(2L,3L)}",
  "{sum<-7L;sum(2L,3L)}", "{sum<-7L;sum}",
  "{s<-sum;f<-function(){sum<-9L;sum(2L,3L)};f()}",
  "{`return`<-function(x)x+1L;f<-function(){return(2L);7L};f()}",
  "{`return`<-7L;f<-function(){return(2L);7L};f()}",
  "{missing<-function(x)FALSE;f<-function(x)missing(x);f()}",
  "{missing<-7L;f<-function(x)missing(x);f()}",
  "{s<-sum;s({cat('A');1L},{cat('B');2L})}",
  "{i<-invisible;i({cat('I');4L})}",
  "{s<-sum;f<-function(){s(return(invisible(8L)));99L};f()}",
  "{r<-`return`;f<-function(){r(invisible(8L));99L};f()}",
  "{s<-sum;for(x in 1:3){s(break)};9L}",
  "{s<-sum;n<-0L;for(x in 1:3){n<-n+1L;s(next);n<-99L};n}",
  "{r<-`return`;f<-function(...)r(...);f(3L)}",
  "{s<-sum;f<-function(...)s(...);f(1L,2L)}",
  "{m<-missing;f<-function(...)m(...);f()}",
  "{m<-missing;f<-function(...)m(...);f(1)}",
  "{m<-missing;f<-function(x){g<-function(y)m(y);g(x)};f()}",
  "{s<-sum;sapply(list(1:2,3:4),s)}",
  "{l<-length;sapply(list(1:2,3:5),l)}",
  "{l<-length;apply(matrix(1:6,2),1,l)}",
  "{s<-sum;sum<-9L;sapply(list(1:2),s)}",
  "{s<-sum;list(s)[[1]](2L,3L)}",
  "{s<-sum;(s)(2L,3L)}",
  "{s<-sum;s()}", "{l<-length;l()}", "{l<-length;l(1,2)}",
  "{i<-invisible;i(1,2)}", "{r<-`return`;r(2L)}",
  "{m<-missing;m(1)}", "{m<-missing;m(x)}",
  "{r<-`return`;f<-function()r(1,2);f()}",
  "{s<-sum;s(stop('FORCED'),{cat('LATE');2})}",
  "{m<-missing;f<-function(x)m(x,stop('FORCED'));f()}",
  "{i<-invisible;i(x=3L)}", "{l<-length;l(x=1:3)}",
  "{s<-sum;typeof(s)}", "{s<-sum;withVisible(s)$visible}",
  "{i<-invisible;withVisible(i(2))$visible}",
  "{s<-sum;withVisible(s(invisible(2)))$visible}",
  "lapply(list(1L),`return`)", "sapply(list(1L),`return`)",
  "apply(matrix(1:4,2),1,`return`)", "lapply(list(1L),missing)",
  "sapply(list(1L),missing)", "apply(matrix(1:4,2),1,missing)",
  "lapply(list(1L),`return`,extra={cat('FORCED');3})",
  "apply(matrix(1:4,2),1,missing,extra={cat('FORCED');3})",
  "{m<-missing;f<-function(...)m(..1);f()}",
  "{m<-missing;f<-function(...)m(..1);f(1)}",
  "{m<-missing;f<-function(x=1){x;m(x)};f()}",
  "{m<-missing;f<-function(x){x;m(x)};f(1)}",
  "{m<-missing;f<-function(x)m('x');f()}",
  "{l<-length;l(y={cat('FORCED');1})}",
  "{i<-invisible;i(y={cat('FORCED');1})}",
  "{m<-missing;m(y={cat('FORCED');1})}",
  "{l<-length;l({cat('A');1},{cat('B');2})}",
  "{i<-invisible;i({cat('A');1},{cat('B');2})}",
  "{r<-`return`;f<-function()r({cat('A');1},{cat('B');2});f()}",
  "{m<-missing;f<-function(x)m({cat('A');x});f()}",
  "{f<-function(x)x;identical(f,sum)}", "typeof(encodeString)",
  "{s<-sum;f<-function(x)function()x;g<-f(s);identical(g(),sum)}"
)
writeLines(sources,file.path(destination,"cases.txt"))
rows <- character()
for (index in seq_along(sources)) {
  prefix <- file.path(destination,sprintf("%03d",index))
  source <- paste0("options(warn=1);.primitive_result<-withVisible({",sources[index],"});\n",
    "cat('\\nVALUE\\n');print(.primitive_result$value);cat('\\nVISIBLE ',.primitive_result$visible,'\\n',sep='');\n",
    ".primitive_value<-.primitive_result$value;cat('TYPE ',typeof(.primitive_value),'\\n',sep='');\n",
    "if(is.primitive(.primitive_value)){.primitive_encoding<-paste0(typeof(.primitive_value),':',deparse(.primitive_value))}",
    "else if(is.null(.primitive_value)){.primitive_encoding<-'NULL'}",
    "else {.primitive_encoding<-paste0(typeof(.primitive_value),':',paste(as.character(.primitive_value),collapse=','))};\n",
    "writeLines(paste(.primitive_encoding,.primitive_result$visible,sep='\\t'),",dQuote(paste0(prefix,".value"),q=FALSE),")")
  writeLines(source,paste0(prefix,".R"))
  status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(paste0(prefix,".R"))),
    stdout=paste0(prefix,".stdout"),stderr=paste0(prefix,".stderr"))
  writeLines(as.character(status),paste0(prefix,".status"))
  if(status == 0L) rows <- c(rows,paste(sources[index],readLines(paste0(prefix,".value")),sep="\t"))
}
writeLines(c(paste0("# ",R.version.string,"; direct top level; fresh process; JIT 0"),rows),file.path(destination,"success.tsv"))
writeLines(c(R.version.string, paste("JIT", Sys.getenv("R_ENABLE_JIT")), paste("cases", length(sources)),paste("successful",length(rows))),file.path(destination,"manifest.txt"))
cat(length(sources),"cases;",length(rows),"successful captures\n")
