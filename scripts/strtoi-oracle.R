options(warn=1)
compiler::enableJIT(0)
args <- commandArgs(TRUE)
hex <- function(x) if (!nzchar(x)) "-" else paste(sprintf("%02x", as.integer(charToRaw(x))),collapse="")
cases <- c(
    'strtoi(c("", " ", "\\t", "+", "-", "0", "+0", "-0", NA_character_))',
    'strtoi(c("10", "010", "0x10", "0Xff", "0b11", "08", "09", "00"))',
    'strtoi(c(" 12", "\\t-12", "\\n+12", "\\r12", "\\v12", "\\f12", "12 ", "12\\t", "1 2"))',
    'strtoi(c("0x", "-0x", "+0x1", "0x+1", "0x-1", "++1", "--1", "+-1", "-+1"))',
    'strtoi(c("2147483647", "2147483648", "-2147483647", "-2147483648", "-2147483649", "999999999999999999999999999999"))',
    'strtoi(c("0x7fffffff", "0x80000000", "-0x7fffffff", "-0x80000000", "017777777777", "020000000000"))',
    'strtoi(c("１２", "١٢", "é", " 12", "−12", "12é", " 12"))',
    'strtoi(NULL)', 'strtoi(character(0))', 'strtoi(integer(0))',
    'strtoi(c(10, 1.2, NA_real_, NaN, Inf, -Inf))',
    'strtoi(c(TRUE,FALSE,NA))', 'strtoi(factor(c("010","12",NA)))',
    'strtoi(list(12,"010",NULL,NA,TRUE))', 'strtoi(list(c(1,2)))',
    'strtoi(matrix(c("010","12"),1))', 'strtoi(c(a="10",b="20"))',
    'strtoi(base=16,x="ff")', 'strtoi(b=16,x="ff")',
    'strtoi(base={cat("B");16},x={cat("X");"ff"})',
    'strtoi(stop("x first"),stop("base second"))',
    'strtoi(character(0),stop("base forced"))',
    'strtoi()', 'strtoi("1",10,stop("unused"))',
    'strtoi(x="1",x="2")',
    '.Internal(strtoi(c("10","11"),10L))',
    '.Internal(strtoi(base=c("10","11"),x=10L))',
    '.Internal(strtoi("10",10))', '.Internal(strtoi("10",integer(0)))',
    '.Internal(strtoi("10",NA_integer_))', '.Internal(strtoi(NULL,10L))',
    '.Internal(strtoi(character(0),1L))',
    '.Internal(strtoi({cat("X");"10"},{cat("B");10L},{cat("E");1}))',
    '.Internal(strtoi("10"))',
    'options(warn=2);strtoi("1","bad")',
    'options(warn=-1);strtoi("1","bad")'
)
for (base in c(0,2:36)) {
    cases <- c(cases, sprintf('strtoi(c("0","10","19","z","Z","0x10","-101","+101","1.0","1e2"),%d)',base))
}
for (base in c('NULL','integer(0)','NA','NA_integer_','NaN','Inf','-Inf','-1','1','37','2.9','-0.9','TRUE','FALSE','"16"','"bad"','"2147483648"','c(16,1)','c("16","bad")','list(16)','list("16", "bad")','factor("10")','matrix(16,1)')) {
    cases <- c(cases, sprintf('strtoi("10",%s)',base), sprintf('strtoi(character(0),%s)',base))
}
for (base in c(2,8,10,16,36)) {
    cases <- c(cases,sprintf('strtoi(c("00000000000000000000000000001","-00000000000000000000000001","  +1","+ 1","1\\n"),%d)',base))
}
con <- file(args[1],"w",encoding="UTF-8")
writeLines(paste0("# ",R.version.string," STRTOI UTF-8 en_US.UTF-8"),con)
for (i in seq_along(cases)) {
    options(warn=1)
    warnings <- character()
    error <- ""
    result <- NULL
    output <- capture.output({
        result <- tryCatch(withCallingHandlers(eval(parse(text=cases[i]),new.env(parent=globalenv())),warning=function(w) {
            if(getOption("warn") >= 2) return()
            if(getOption("warn") >= 0) warnings <<- c(warnings,conditionMessage(w))
            invokeRestart("muffleWarning")
        }),error=function(e) {error <<- conditionMessage(e); NULL})
    })
    kind <- if(nzchar(error)) "error" else typeof(result)
    stopifnot(kind %in% c("error","integer"))
    values <- if(kind=="error" || length(result)==0) "-" else paste(ifelse(is.na(result),"NA",result),collapse=",")
    stdout <- if(length(output)) paste0(paste(output,collapse="\n"),"\n") else ""
    if(length(output) && !grepl("\\n",cases[i],fixed=TRUE)) stdout <- sub("\n$","",stdout)
    writeLines(paste(i,cases[i],kind,values,hex(error),hex(paste(warnings,collapse="\n")),hex(stdout),sep="\t"),con)
}
close(con)
cat(length(cases),"oracle rows\n")
