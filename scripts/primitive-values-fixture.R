args <- commandArgs(TRUE)
directory <- args[1]
sources <- readLines(file.path(directory,"cases.txt"))
hex <- function(value) paste(sprintf("%02x", as.integer(charToRaw(value))),collapse="")
rows <- character()
for(index in seq_along(sources)) {
  prefix <- file.path(directory,sprintf("%03d",index))
  output <- readChar(paste0(prefix,".stdout"), file.info(paste0(prefix,".stdout"))$size)
  if(readLines(paste0(prefix,".status")) == "0") {
    before <- strsplit(output,"\nVALUE\n",fixed=TRUE)[[1]][1]
    rows <- c(rows,paste(sources[index],"OK",readLines(paste0(prefix,".value")),hex(before),sep="\t"))
  } else {
    stderr <- readLines(paste0(prefix,".stderr"))
    message <- paste(stderr[seq_len(which(grepl("^(Calls:|Execution halted)",stderr))[1]-1L)],collapse=" ")
    message <- sub("^Error.*? :[[:space:]]*", "",message,perl=TRUE)
    rows <- c(rows,paste(sources[index],"ERR",message,"-",hex(output),sep="\t"))
  }
}
writeLines(rows,"crates/r_ust_eval/tests/fixtures/primitive-values-r-4.6.0.tsv")
cat(length(rows),"outcome fixtures\n")
