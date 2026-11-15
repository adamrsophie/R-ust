// Pinned R 4.6.0 base sources; GPL-2.0-or-later.
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        ".deparseOpts",
        r###"function(control) {
    if(!length(control)) return(0) # fast exit
    opts <- pmatch(as.character(control), ..deparseOpts)
    if (anyNA(opts))
        stop(sprintf(ngettext(as.integer(sum(is.na(opts))),
                              "deparse option %s is not recognized",
                              "deparse options %s are not recognized"),
                     paste(sQuote(control[is.na(opts)]), collapse=", ")),
             call. = FALSE, domain = NA)
    if(any(opts == 1L)) { # "all" now == former ("all", "digits17") -- remain compatible
        if(any(opts == 13L))
            stop('"all" and "exact" are mutually exclusive')
        ## ensuring  c("all", "hexNumeric") does not give error below:
        opts <- unique(c(opts[opts != 1L], 2:6, 8L, if(!any(opts == 10L)) 11L,
                         12L)) # not {7,9} + 10 *or* 11
    } else if(any(opts == 13L)) { ## "exact" := ("all", "hexNumeric")
        opts <- unique(c(opts[opts != 13L], 2:6, 8L, 10L, 12L))
    }
    if(10L %in% opts && 11L %in% opts)
        stop('"hexNumeric" and "digits17" are mutually exclusive')
    sum(2^(opts-2))
}"###,
    ),
    (
        "deparse",
        r###"function(expr, width.cutoff = 60L,
	     backtick = mode(expr) %in% c("call", "expression", "(", "function"),
	     control = c("keepNA", "keepInteger", "niceNames", "showAttributes"),
             nlines = -1L)
    .Internal(deparse(expr, width.cutoff, backtick,
                      .deparseOpts(control), nlines))"###,
    ),
];
