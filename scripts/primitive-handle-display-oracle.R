# R 4.6.0 primitive display metadata; supplementary observations for the five handle sources.
# Rscript --vanilla SCRIPT OUTPUT.tsv SCRATCH_DIRECTORY
stopifnot(getRversion() == "4.6.0")
Sys.setenv(R_ENABLE_JIT="0")
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
out <- args[1L]
scratch <- args[2L]
dir.create(scratch,recursive=TRUE,showWarnings=FALSE)
sources <- c("sum","length","invisible","`return`","missing")
rows <- character()
for(index in seq_along(sources)) {
  stem <- file.path(scratch,sprintf("%03d",index))
  result <- normalizePath(paste0(stem,".tsv"),mustWork=FALSE)
  code <- c(
    "stopifnot(getRversion() == '4.6.0');invisible(compiler::enableJIT(0));options(width=80)",
    paste0(".profile_answer<-withVisible({",sources[index],"})"),
    ".profile_value<-.profile_answer$value",
    ".profile_output<-paste0(paste(capture.output(print(.profile_value)),collapse='\\n'),'\\n')",
    ".profile_hex<-paste(sprintf('%02x',as.integer(charToRaw(.profile_output))),collapse='')",
    paste0("writeLines(paste(",encodeString(sources[index],quote='"'),",typeof(.profile_value),paste(class(.profile_value),collapse=','),is.function(.profile_value),length(.profile_value),.profile_answer$visible,.profile_hex,sep='\\t'),",encodeString(result,quote='"'),")")
  )
  writeLines(code,paste0(stem,".R"))
  status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(paste0(stem,".R"))),stdout=paste0(stem,".stdout"),stderr=paste0(stem,".stderr"))
  stopifnot(status == 0L)
  row <- readLines(result)
  stopifnot(length(row)==1L)
  rows <- c(rows,row)
}
dir.create(dirname(out),recursive=TRUE,showWarnings=FALSE)
writeLines(c("source\tkind\tclass\tis_function\tlength\tvisible\tdisplay_hex",rows),out)
cat("5 supplemental primitive metadata observations; no additional source cases\n")
