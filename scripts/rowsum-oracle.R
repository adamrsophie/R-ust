stopifnot(getRversion() == "4.6.0")
stopifnot(Sys.setlocale("LC_COLLATE", "C") == "C")
cat(R.version.string, "LC_COLLATE=", Sys.getlocale("LC_COLLATE"), "\n")
cases <- c(
"rowsum(matrix(1:6,3),c(2,1,2))",
"rowsum(matrix(1:6,3),c(2,1,2),FALSE)",
"rowsum(matrix(c(1,2,4,8,16,32),3),c('b','a','b'))",
"rowsum(matrix(1:6,3),factor(c('a','b','a'),levels=c('b','a')))",
"rowsum(1:4,c(2,NA,1,NA))",
"rowsum(1:4,c(2,NA,1,NA),FALSE)",
"rowsum(c(1,NaN,NA_real_,4),c(1,1,2,2))",
"rowsum(c(1,NaN,NA_real_,4),c(1,1,2,2),na.rm=TRUE)",
"rowsum(c(1L,NA_integer_,3L),c(1,1,2))",
"rowsum(c(1L,NA_integer_,3L),c(1,1,2),na.rm=TRUE)",
"rowsum(c(2147483647L,1L,-1L),c(1,1,1))",
"rowsum(c(-2147483647L,-1L,1L),c(1,1,1))",
"rowsum(data.frame(a=1:3,b=c(1,2,4)),c(2,1,2))",
"rowsum(data.frame(a=1:3,b=c(1,2,4)),c(2,1,2),FALSE)",
"rowsum(matrix(integer(0),0,2),integer(0))",
"rowsum(matrix(integer(0),3,0),1:3)",
"rowsum(1:3,c(TRUE,FALSE,TRUE))",
"rowsum(1:3,c(Inf,-Inf,Inf))",
"rowsum(1:4,c(NA_real_,NaN,NA_real_,NaN))",
"rowsum(x=1:3,gr=c(2,1,2),re=FALSE)",
"rowsum(1:3,c(2,1,2),unused=stop('unused forced'))",
"rowsum(c(1L,NA_integer_,3L),c(2,1,2),na.r=TRUE)",
"rowsum.default(TRUE,stop('group forced'))",
"rowsum(1:3,1:2,reorder=stop('reorder forced'))",
"rowsum(1:3,1:3,reorder=NA)",
"rowsum(1:3,1:3,na.rm=NA)",
"rowsum(1:3,1:3,na.rm=logical(0))",
"rowsum(1:3,1:3,na.rm=c(TRUE,FALSE))",
"rowsum(data.frame(a=letters[1:3]),1:3)",
"rowsum.data.frame(1:3,stop('group forced'))",
"rowsum(x=1:3,x=2:4,group=1:3)",
"rowsum(1:3)",
"rowsum()",
"rowsum(1:4,c('Z','a','ä','A'))",
"rowsum(1:4,c('Z','a','ä','A'),FALSE)",
".rowsum.check(1,1)",
".Internal(.rowsum.sum(1,1,TRUE,FALSE))",
"matrix(NULL,0,2)",
"matrix(logical(0),0,2)",
"matrix(integer(0),0,2)",
"matrix(double(0),0,2)",
"matrix(character(0),0,2)",
"matrix(list(),0,2)",
"matrix(factor(character(0)),0,2)"
)
atom <- function(x) paste(ifelse(is.na(x),ifelse(is.nan(x),'NaN','NA'),as.character(x)),collapse=',')
canonical <- function(x) {
 if(is.data.frame(x)) return(paste0('df:',nrow(x),':',paste(vapply(x,function(v) paste0(typeof(v),':',atom(v)),''),collapse=';')))
 paste0(typeof(x),':',paste(dim(x),collapse=','),':',atom(x))
}
rows <- lapply(seq_along(cases),function(i) {
 warnings <- character()
 error <- '-'
 value <- tryCatch(withCallingHandlers(eval(parse(text=cases[i])),warning=function(w) {warnings <<- c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e) {error <<- conditionMessage(e);NULL})
 paste(i,cases[i],if(error=='-') canonical(value) else '-',error,if(length(warnings)) paste(warnings,collapse='|') else '-',sep='\t')
})
writeLines(c('id\tsource\tvalue\terror\twarning',unlist(rows)), 'crates/r_ust_eval/tests/fixtures/rowsum-r-4.6.0.tsv')

column_names <- lapply(c(13L,14L),function(i) {
 value <- eval(parse(text=cases[i]))
 labels <- names(value)
 values <- vapply(labels,function(label) if(is.na(label)) "NA" else paste0("x",paste(format(charToRaw(label)),collapse="")),"")
 fingerprint <- paste0("character|",length(labels),"|",paste(values,collapse=","))
 paste(i,cases[i],fingerprint,sep="\t")
})
writeLines(c(paste0("# ",R.version.string," LC_COLLATE=C rowsum column names"),
 "id\tsource\tnames",unlist(column_names)),
 "crates/r_ust_eval/tests/fixtures/rowsum-column-names-r-4.6.0.tsv")
