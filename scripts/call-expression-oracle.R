stopifnot(getRversion() == "4.6.0")
Sys.setenv(R_ENABLE_JIT="0")
args <- commandArgs(TRUE)
destination <- args[1]
candidate <- normalizePath(args[2])
if(dir.exists(destination)) stop("Refusing to replace existing captures")
dir.create(destination,recursive=TRUE)
sources <- c(
  "{sum<-7L;(sum)(1L,2L)}",
  "{sum<-7L;sum(1L,2L)}",
  "{length<-9L;(length)(1:3)}",
  "{length<-9L;length(1:3)}",
  "{sin<-7L;(sin)(0)}",
  "{sin<-7L;sin(0)}",
  "(sin)(0)",
  "((sin))(0)",
  "(((sum)))(1L,2L)",
  "{sum<-7L;((sum))({cat(\"ARG\");1L})}",
  "{sin<-7L;((sin))({cat(\"ARG\");0})}",
  "{f<-function(){sum<-7L;(sum)({cat(\"ARG\");1L})};f()}",
  "{f<-function(sum=7L)(sum)({cat(\"ARG\");1L});f()}",
  "{f<-function(sin=7L)(sin)({cat(\"ARG\");0});f()}",
  "{f<-function(fun)(fun)(1L,2L);f(sum)}",
  "{f<-function(fun=sum)(fun)(1L,2L);f()}",
  "{f<-function(fun={cat(\"FUN\");sum})(fun)({cat(\"A\");1L},{cat(\"B\");2L});f()}",
  "{f<-function(fun={cat(\"FUN\");7L})(fun)({cat(\"ARG\");1L});f()}",
  "{f<-function(fun)(fun)({cat(\"ARG\");1L});f({cat(\"FUN\");7L})}",
  "{f<-function(fun={cat(\"FUN\");sum}){(fun)(1L);(fun)(2L)};f()}",
  "{sum<-function(x)x+1L;(sum)(2L)}",
  "{sin<-function(x)x+1L;((sin))(2L)}",
  "{f<-function(x)x+1L;g<-function(f=7L)(f)({cat(\"ARG\");2L});g()}",
  "{f<-function(x)x+1L;g<-function(f=7L)f(2L);g()}",
  "{s<-sum;(s)(1L,2L)}",
  "{l<-length;((l))(1:3)}",
  "{i<-invisible;(i)(2L)}",
  "{r<-`return`;f<-function(){(r)(invisible(8L));99L};f()}",
  "{m<-missing;f<-function(x)(m)(x);f()}",
  "{m<-missing;f<-function(x)(m)(x);f(1L)}",
  "{f<-function(x,fun=invisible(missing))(fun)(x);f()}",
  "{s<-sum;({cat(\"FUN\");s})({cat(\"A\");1L},{cat(\"B\");2L})}",
  "{({cat(\"FUN\");7L})({cat(\"ARG\");1L})}",
  "{f<-function()sum;(f())(1L,2L)}",
  "{f<-function()7L;(f())({cat(\"ARG\");1L})}",
  "{(function(x)x+1L)(2L)}",
  "{f<-function(fun)function(x)(fun)(x);f(length)(1:3)}",
  "{f<-function(){(sum)(return(invisible(8L)));99L};f()}",
  "{for(x in 1:3){(sum)(break)};9L}",
  "{n<-0L;for(x in 1:3){n<-n+1L;(sum)(next);n<-99L};n}",
  "{f<-function(){(return(sum))({cat(\"ARG\");1L});99L};identical(f(),sum)}",
  "{for(x in 1:3){(break)({cat(\"ARG\");1L})};9L}",
  "{n<-0L;for(x in 1:3){n<-n+1L;(next)({cat(\"ARG\");1L});n<-99L};n}",
  "withVisible((invisible(1L)))$visible",
  "{f<-function(fun=invisible(sum))(fun)(1L,2L);f()}",
  "{f<-function(fun=invisible(invisible))(fun)(2L);f()}",
  "(.Internal)(utf8ToInt(\"A\"))",
  "((.Internal))(utf8ToInt(\"A\"))",
  "{.Internal<-7L;(.Internal)(utf8ToInt({cat(\"ARG\");\"A\"}))}",
  "{.Internal<-7L;.Internal(utf8ToInt(\"A\"))}",
  "{f<-function(...)(...length)();f(1L,2L)}",
  "{f<-function(...)(((...length)))();f(1L,2L)}",
  "{f<-function(...){...length<-7L;(...length)()};f(1L,2L)}",
  "{f<-function(...){...length<-7L;...length()};f(1L,2L)}"
)
raw <- function(path) readBin(path,"raw",file.info(path)$size)
hex <- function(value) if(length(value)) paste(sprintf("%02x",as.integer(value)),collapse="") else "-"
writeLines(sources,file.path(destination,"cases.txt"))
rows <- character()
comparisons <- character()
for(index in seq_along(sources)) {
  prefix <- file.path(destination,sprintf("%03d",index))
  body <- paste0("options(warn=1);.call_result<-withVisible({",sources[index],"});\n",
    "cat('\\nVALUE\\n');print(.call_result$value);cat('\\nVISIBLE ',.call_result$visible,'\\n',sep='');\n")
  metadata <- paste0("writeLines(paste(paste0(typeof(.call_result$value),':',paste(as.character(.call_result$value),collapse=',')),.call_result$visible,sep='\\t'),",
    dQuote(paste0(prefix,".value"),q=FALSE),")")
  writeLines(c(body,metadata),paste0(prefix,".reference.R"))
  writeLines(body,paste0(prefix,".candidate.R"))
  reference_status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(paste0(prefix,".reference.R"))),stdout=paste0(prefix,".reference.stdout"),stderr=paste0(prefix,".reference.stderr"),timeout=15)
  candidate_status <- system2(candidate,c("--vanilla",shQuote(paste0(prefix,".candidate.R"))),stdout=paste0(prefix,".candidate.stdout"),stderr=paste0(prefix,".candidate.stderr"),timeout=15)
  writeLines(as.character(reference_status),paste0(prefix,".reference.status"))
  writeLines(as.character(candidate_status),paste0(prefix,".candidate.status"))
  output <- rawToChar(raw(paste0(prefix,".reference.stdout")))
  if(reference_status == 0L) {
    before <- strsplit(output,"\nVALUE\n",fixed=TRUE)[[1]][1]
    rows <- c(rows,paste(sources[index],"OK",readLines(paste0(prefix,".value")),hex(charToRaw(before)),sep="\t"))
  } else {
    error <- rawToChar(raw(paste0(prefix,".reference.stderr")))
    stopifnot(reference_status == 1L,grepl("attempt to apply non-function",error,fixed=TRUE))
    rows <- c(rows,paste(sources[index],"ERR","attempt to apply non-function","-",hex(charToRaw(output)),sep="\t"))
  }
  matched <- reference_status == candidate_status && identical(raw(paste0(prefix,".reference.stdout")),raw(paste0(prefix,".candidate.stdout"))) && identical(raw(paste0(prefix,".reference.stderr")),raw(paste0(prefix,".candidate.stderr")))
  comparisons <- c(comparisons,paste(index,reference_status,candidate_status,matched,sep="\t"))
}
writeLines(rows,"crates/r_ust_eval/tests/fixtures/call-expression-r-4.6.0.tsv")
writeLines(comparisons,file.path(destination,"blocked-comparison.tsv"))
writeLines(c(R.version.string,"Fresh direct top-level child per case; R_ENABLE_JIT=0",candidate),file.path(destination,"manifest.txt"))
cat(length(rows),"reference cases;",sum(endsWith(comparisons,"FALSE")),"blocked candidate mismatches\n")
