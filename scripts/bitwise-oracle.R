stopifnot(getRversion() == "4.6.0")
args <- commandArgs(TRUE)
stopifnot(length(args) == 2L)
out <- args[[1L]]
scratch <- args[[2L]]
stopifnot(!dir.exists(scratch))
dir.create(scratch, recursive=TRUE)
cases <- character()
add <- function(name, source) {
    stopifnot(!name %in% names(cases), !grepl("[\t\r\n]", source))
    cases[[name]] <<- source
}
binary <- c("bitwAnd", "bitwOr", "bitwXor", "bitwShiftL", "bitwShiftR")
for (fun in binary) {
    for (pair in list(c("c(0L,1L,2L,3L,-1L,-2L,2147483647L,NA_integer_)", "1L"),
                     c("c(1L,2L,4L,8L,16L)", "c(1L,2L)"),
                     c("integer()", "1L"), c("1L", "integer()"),
                     c("c(1.9,-1.9,2147483647.9,-2147483647.9)", "2L"),
                     c("c(NA_real_,NaN,Inf,-Inf,2147483648,-2147483648)", "1L"),
                     c("factor(c('b','a',NA))", "2L"),
                     c("matrix(c(1L,2L,3L,NA_integer_),2L)", "2L"),
                     c("c(a=1L,b=2L)", "c(c=3L,d=4L)"))) {
        add(paste0(fun,"_value_",length(cases)+1L), paste0(fun,"(",pair[[1L]],",",pair[[2L]],")"))
    }
    for (pair in list(c("TRUE","FALSE"),c("TRUE","1L"),c("1L","TRUE"),
                     c("'1'","'2'"),c("'1'","1L"),c("1L","'2'"),
                     c("NULL","NULL"),c("NULL","1L"),c("1L","NULL"),
                     c("list(1L)","list(2L)"),c("list(1L)","1L"),c("1L","list(2L)"),
                     c("function()1L","function()2L"),c("1L","function()2L"),
                     c("integer()","'bad'"),c("Inf","'bad'"),c("'bad'","Inf"))) {
        add(paste0(fun,"_type_",length(cases)+1L), paste0(fun,"(",pair[[1L]],",",pair[[2L]],")"))
    }
    second <- if (fun %in% c("bitwShiftL","bitwShiftR")) "n" else "b"
    for (tail in c("()","(1L)","(,1L)","(1L,)","(1L,2L,extra=stop('unused'))",
                   "(a=1L,a=stop('duplicate'),2L)","(a={cat('A');1L},extra={cat('X');2L})"))
        add(paste0(fun,"_arity_",length(cases)+1L),paste0(fun,tail))
    add(paste0(fun,"_named_order"),paste0(fun,"(",second,"={cat('B');1L},a={cat('A');3L})"))
    add(paste0(fun,"_force_before_coercion"),paste0(fun,"(a=Inf,",second,"={cat('B');1L})"))
    add(paste0(fun,"_warn2_order"),paste0("options(warn=2);",fun,"(a=Inf,",second,"={cat('B');1L})"))
    add(paste0(fun,"_warn2_second"),paste0("options(warn=2);",fun,"(1L,Inf)"))
    add(paste0(fun,"_two_warnings"),paste0(fun,"(Inf,Inf)"))
    add(paste0(fun,"_warning_disabled"),paste0("options(warn=-1);",fun,"(Inf,1L)"))
    add(paste0(fun,"_alias"),paste0("f<-",fun,";f(3L,1L)"))
    add(paste0(fun,"_shadow"),paste0("f<-function(",fun,")",fun,"(3L,1L);f(function(a,b)17L)"))
    for (position in c("a",second)) {
        for (invisible in c(FALSE,TRUE)) {
            transfer <- if(invisible) "return(invisible(17L))" else "return(17L)"
            arguments <- if(position=="a") paste0("{cat('A');",transfer,"},{cat('B');1L}") else paste0("{cat('A');Inf},{cat('B');",transfer,"}")
            add(paste0(fun,"_return_",position,"_",invisible),paste0("(function(){",fun,"(",arguments,");99L})()"))
        }
        for (transfer in c("break","next")) {
            arguments <- if(position=="a") paste0("{cat('A');",transfer,"},{cat('B');1L}") else paste0("{cat('A');Inf},{cat('B');",transfer,"}")
            add(paste0(fun,"_loop_",position,"_",transfer),paste0("n<-0L;for(i in 1:2){",fun,"(",arguments,");n<-n+1L};c(n,i)"))
        }
    }
    internal <- sub("bitw", "bitwise", fun, fixed=TRUE)
    add(paste0(fun,"_internal"),paste0(".Internal(",internal,"(3L,1L))"))
    add(paste0(fun,"_internal_arity_effect"),paste0(".Internal(",internal,"({cat('A');1L},{cat('B');2L},{cat('C');3L}))"))
}
for (value in c("c(0L,1L,-1L,-2L,2147483647L,-2147483647L,NA_integer_)","integer()",
                "c(1.9,-1.9,NA_real_,NaN,Inf,-Inf)","TRUE","logical()","NULL","'1'",
                "list(1L)","function()1L","factor(c('b','a',NA))","matrix(1:4,2L)","c(a=1L,b=2L)"))
    add(paste0("not_value_",length(cases)+1L),paste0("bitwNot(",value,")"))
for (source in c("bitwNot()","bitwNot(a=1L,a=stop('duplicate'))","bitwNot(1L,stop('unused'))",
                 "bitwNot(a={cat('A');1L})","options(warn=2);bitwNot(Inf)",
                 "(function(){bitwNot(return(invisible(17L)));99L})()",
                 "n<-0L;for(i in 1:2){bitwNot(next);n<-n+1L};c(n,i)",
                 ".Internal(bitwiseNot({cat('A');1L},{cat('B');2L}))"))
    add(paste0("not_contract_",length(cases)+1L),source)
for (fun in c("bitwShiftL","bitwShiftR")) {
    for (count in c("c(-1L,0L,1L,30L,31L,32L,NA_integer_)","c(-0.9,31.9,32,Inf,NaN)",
                    "c('0','1','31','bad','NaN','2147483648')","c(TRUE,FALSE,NA)",
                    "factor(c('b','a',NA))","list(1L,'bad','bad',Inf)","list(NULL)",
                    "list(1:2)","list(list(1L))","data.frame(n=c(0L,1L))", "matrix(c(0L,31L),1L)"))
        add(paste0(fun,"_counts_",length(cases)+1L),paste0(fun,"(-1L,",count,")"))
    add(paste0(fun,"_high_bits"),paste0(fun,"(c(1L,2L,3L,-1L,-2L,-2147483647L),31L)"))
}
add("and_na_dominates_zero","bitwAnd(c(NA_integer_,1L),c(0L,NA_integer_))")
add("or_na_dominates_ones","bitwOr(c(NA_integer_,1L),c(-1L,NA_integer_))")
add("xor_na_result","bitwXor(-1L,2147483647L)")
hex <- function(text) if (!nzchar(text)) "-" else paste(format(charToRaw(text)), collapse="")
quote <- function(text) encodeString(text, quote='"')
header <- "id\tname\tsource\ttype\tvalues\tvisible\terror_hex\twarnings_hex\tstdout_hex"
writeLines(header,out)
for(i in seq_along(cases)) {
    dir <- file.path(scratch,sprintf("%03d",i));dir.create(dir)
    program <- c('stopifnot(getRversion()=="4.6.0");options(warn=1)',
        '.bw_warnings<-character();.bw_error<-""',
        paste0('.bw_con<-file(',quote(file.path(dir,"output.raw")),',"wb");sink(.bw_con)'),
        '.bw_result<-tryCatch(withCallingHandlers(withVisible({',unname(cases[[i]]),
        '}),warning=function(w){if(getOption("warn")>=2)return(invisible(NULL));if(getOption("warn")>=0).bw_warnings<<-c(.bw_warnings,conditionMessage(w));invokeRestart("muffleWarning")}),error=function(e){.bw_error<<-conditionMessage(e);NULL})',
        'sink();close(.bw_con)',
        'hex<-function(x)if(!nzchar(x))"-"else paste(format(charToRaw(x)),collapse="")',
        '.bw_kind<-if(nzchar(.bw_error))"error"else typeof(.bw_result$value)',
        '.bw_values<-if(.bw_kind=="error"||!length(.bw_result$value))"-"else paste(ifelse(is.na(.bw_result$value),"NA",as.character(.bw_result$value)),collapse=",")',
        '.bw_visible<-if(.bw_kind=="error")"-"else if(.bw_result$visible)"TRUE"else"FALSE"',
        paste0('writeLines(paste(.bw_kind,.bw_values,.bw_visible,hex(.bw_error),hex(paste(.bw_warnings,collapse="\\n")),sep="\\t"),',quote(file.path(dir,"result.tsv")),')'))
    path <- file.path(dir,"case.R");writeLines(program,path)
    status <- system2(file.path(R.home("bin"),"Rscript"),c("--vanilla",shQuote(path)),
        stdout=file.path(dir,"process.stdout"),stderr=file.path(dir,"process.stderr"),
        env=c("R_ENABLE_JIT=0","R_DEFAULT_PACKAGES=NULL","LC_ALL=en_US.UTF-8"))
    stopifnot(status==0L, file.info(file.path(dir,"process.stdout"))$size==0,
              file.info(file.path(dir,"process.stderr"))$size==0)
    result <- readLines(file.path(dir,"result.tsv"));stopifnot(length(result)==1L)
    raw <- readBin(file.path(dir,"output.raw"),"raw",n=file.info(file.path(dir,"output.raw"))$size)
    cat(paste(i,names(cases)[[i]],cases[[i]],result,if(length(raw))paste(format(raw),collapse="")else"-",sep="\t"),"\n",sep="",file=out,append=TRUE)
}
cat("R oracle rows:",length(cases),"\n")
