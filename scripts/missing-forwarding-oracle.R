stopifnot(getRversion() == "4.6.0")
Sys.setenv(R_ENABLE_JIT="0")
args <- commandArgs(TRUE)
destination <- args[1]
candidate <- normalizePath(args[2])
if(dir.exists(destination)) stop("Refusing to replace existing reference captures")
dir.create(destination,recursive=TRUE)
sources <- c(
  "{g<-function(y)missing(y);f<-function(x=1L)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x=1L){x;g(x)};f()}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);f()}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);f(,2L)}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);f(1L,2L)}",
  "{f<-function(x=1L)missing(x);f()}",
  "{f<-function(x=1L){x;missing(x)};f()}",
  "{g<-function(y)missing(y);f<-function(x)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x)g(x);f(1L)}",
  "{g<-function(y)missing(y);f<-function(x=1L)c(missing(x),g(x));f()}",
  "{g<-function(y)missing(y);f<-function(x=y,y)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x=y,y=2L)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x=y,y)g(x);f(y=2L)}",
  "{g<-function(y)missing(y);f<-function(x=absent)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x=absent)c(missing(x),g(x));f()}",
  "{g<-function(y)missing(y);f<-function(x)g(x);f(absent)}",
  "{g<-function(y)missing(y);f<-function(x=y,y=2L){x;c(missing(x),g(x))};f()}",
  "{g<-function(y){y;missing(y)};f<-function(x=1L)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x={cat('DEFAULT');1L})g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x={cat('DEFAULT');1L}){x;g(x)};f()}",
  "{g<-function(y)missing(y);f<-function(x=stop('FORCED'))g(x);f()}",
  "{g<-function(y)missing(y);f<-function(...)g(..2);f(1L)}",
  "{g<-function(y)missing(y);f<-function(...)g(..2);f(1L,)}",
  "{g<-function(y)missing(y);f<-function(...)g(..2);f(1L,2L)}",
  "{g<-function(y)missing(y);f<-function(...){..1;g(..1)};f(1L)}",
  "{g<-function(y)missing(y);f<-function(...)g((..1));f()}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);h<-function(x)f(x);h()}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);h<-function(x=1L)f(x);h()}",
  "{g<-function(y)missing(y);f<-function(...)g(..1);h<-function(x=1L){x;f(x)};h()}",
  "{m<-missing;g<-function(y)m(y);f<-function(x=1L)g(x);f()}",
  "{m<-missing;g<-function(y)m(y);f<-function(...)g(..1);f()}",
  "{m<-missing;g<-function(y)m(y);f<-function(...)g(..1);f(,2L)}",
  "{m<-missing;g<-function(y)m(y);f<-function(x=y,y=2L)c(m(x),g(x));f()}",
  "{g<-function(y)missing(y);f<-function(x=y,y=x)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(x=x)g(x);f()}",
  "{g<-function(y)missing(y);f<-function(...)g(...);h<-function(x=1L)f(x);h()}",
  "{g<-function(y)missing(y);f<-function(...)g(...);h<-function(x)f(x);h()}",
  "{g<-function(y)missing(y);f<-function(x=1L){x<-2L;c(missing(x),g(x))};f()}"
)
hex <- function(value) if(nzchar(value)) paste(sprintf("%02x",as.integer(charToRaw(value))),collapse="") else "-"
raw <- function(path) readBin(path,"raw",file.info(path)$size)
writeLines(sources,file.path(destination,"cases.txt"))
rows <- character()
comparisons <- character()
for(index in seq_along(sources)) {
  prefix <- file.path(destination,sprintf("%03d",index))
  body <- paste0("options(warn=1);.repair_result<-withVisible({",sources[index],"});\n",
    "cat('\\nVALUE\\n');print(.repair_result$value);cat('\\nVISIBLE ',.repair_result$visible,'\\n',sep='');\n")
  metadata <- paste0("writeLines(paste(paste0(typeof(.repair_result$value),':',paste(as.character(.repair_result$value),collapse=',')),.repair_result$visible,sep='\\t'),",
    dQuote(paste0(prefix,".value"),q=FALSE),")")
  writeLines(c(body,metadata),paste0(prefix,".reference.R"))
  writeLines(body,paste0(prefix,".candidate.R"))
  reference_status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(paste0(prefix,".reference.R"))),
    stdout=paste0(prefix,".reference.stdout"),stderr=paste0(prefix,".reference.stderr"),timeout=15)
  candidate_status <- system2(candidate,c("--vanilla","--no-echo","-f",shQuote(paste0(prefix,".candidate.R"))),
    stdout=paste0(prefix,".candidate.stdout"),stderr=paste0(prefix,".candidate.stderr"),timeout=15)
  writeLines(as.character(reference_status),paste0(prefix,".reference.status"))
  writeLines(as.character(candidate_status),paste0(prefix,".candidate.status"))
  stopifnot(reference_status == 0L)
  output <- rawToChar(raw(paste0(prefix,".reference.stdout")))
  before <- strsplit(output,"\nVALUE\n",fixed=TRUE)[[1]][1]
  stderr <- rawToChar(raw(paste0(prefix,".reference.stderr")))
  rows <- c(rows,paste(sources[index],readLines(paste0(prefix,".value")),hex(before),hex(stderr),sep="\t"))
  matched <- reference_status == candidate_status &&
    identical(raw(paste0(prefix,".reference.stdout")),raw(paste0(prefix,".candidate.stdout"))) &&
    identical(raw(paste0(prefix,".reference.stderr")),raw(paste0(prefix,".candidate.stderr")))
  comparisons <- c(comparisons,paste(index,reference_status,candidate_status,matched,sep="\t"))
}
writeLines(rows,"crates/r_ust_eval/tests/fixtures/missing-forwarding-r-4.6.0.tsv")
writeLines(comparisons,file.path(destination,"blocked-comparison.tsv"))
writeLines(c(R.version.string,"Fresh direct top-level child per case; R_ENABLE_JIT=0",candidate),file.path(destination,"manifest.txt"))
cat(length(rows),"reference cases;",sum(endsWith(comparisons,"FALSE")),"blocked candidate mismatches\n")
