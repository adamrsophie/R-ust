# Version 1 bounded structural transport; observation starts after the subject.
df_hex <- function(x) {
    if (!nzchar(x)) return("_")
    paste(sprintf("%02x", as.integer(charToRaw(enc2utf8(x)))), collapse="")
}
df_node <- function(x) {
    if (missing(x) || identical(x, quote(expr=))) {
        cat("missing\t0\t-\t_\t0\nend\n")
        return(invisible(NULL))
    }
    tag <- typeof(x)
    count <- length(x)
    children <- 0L
    payload <- "_"
    labels <- names(x)
    if (is.null(labels)) labels <- "-" else labels <- paste(vapply(labels, df_hex, ""), collapse=",")
    if (tag == "closure") {
        count <- 1L
        children <- 3L
    } else if (tag == "environment") {
        labels <- "-"
        count <- 1L
        payload <- df_hex(paste(c(identical(x,globalenv()), identical(x,baseenv()),
            identical(x,.BaseNamespaceEnv), identical(x,emptyenv())),collapse=","))
    } else if (tag %in% c("list","pairlist","language")) {
        children <- count
    } else if (tag == "symbol") {
        count <- 1L
        payload <- df_hex(as.character(x))
    } else if (tag %in% c("logical","integer","double","character")) {
        payload <- paste(vapply(seq_along(x), function(i) {
            if (is.na(x[i])) return("NA")
            if (tag == "character") return(df_hex(x[i]))
            if (tag == "double") return(sprintf("%a",x[i]))
            as.character(as.integer(x[i]))
        }, ""),collapse=",")
        if (!count) payload <- "_"
    } else if (tag != "NULL") stop("unsupported structural type: ",tag)
    cat(tag,count,labels,payload,children,sep="\t")
    cat("\n")
    if (tag == "closure") {
        df_node(formals(x)); df_node(body(x)); df_node(environment(x))
    } else if (children) {
        for (i in seq_len(children)) df_node(x[[i]])
    }
    cat("end\n")
}
cat("DFNODE1\n")
cat(if (oracle_result$visible) "visible\n" else "invisible\n")
df_node(oracle_result$value)
