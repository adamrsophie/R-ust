stopifnot(getRversion() == "4.6.0", .Machine$sizeof.longdouble == 8L)
stopifnot(Sys.setlocale("LC_ALL", "C") != "")
options(digits=7, scipen=0, width=80)
destination <- commandArgs(TRUE)[1L]
stopifnot(!file.exists(destination))
values <- c("1.2345678901234567", "100000", "0.0001", "0.00001",
            "1e100", "1e-100", "NA_real_", "NaN", "Inf", "-Inf",
            "-0.0", "999999999999999.5")
rows <- vapply(values, function(value) {
  source <- paste0("g<-function()NULL;f<-function(...)g(...);f(x=", value, ")")
  message <- tryCatch({eval(parse(text=source), new.env()); stop("Expected unused argument")},
                      error=conditionMessage)
  stopifnot(startsWith(message, "unused argument (x = "))
  encoded <- paste(sprintf("%02x", as.integer(charToRaw(message))), collapse="")
  paste(source, encoded, sep="\t")
}, character(1))
writeLines(c("source\terror_hex", rows), destination)
cat(length(rows), "native R unused-argument diagnostic references\n")
