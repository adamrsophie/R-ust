// Pinned R 4.6.0 base sources; GPL-2.0-or-later.
pub(super) const DEFINITIONS: &[(&str, &str)] = &[
    (
        "sQuote",
        r###"function(x, q = getOption("useFancyQuotes"))
{
    if (!length(x)) return(character())
    before <- after <- "'"
    if(!is.null(q)) {
        if(isTRUE(q)) {
            li <- l10n_info()
            if(li$"UTF-8") q <- "UTF-8"
            if(!is.null(li$codepage) && li$codepage > 0L) {
                ## we can't just use iconv, as that seems to think
                ## it is in latin1 in CP1252
                if(li$codepage >= 1250L && li$codepage <= 1258L
                   || li$codepage == 874L) {
                    before <- rawToChar(as.raw(0x91))
                    after <- rawToChar(as.raw(0x92))
                } else {
                    z <- iconv(c("\u2018", "\u2019"), "UTF-8", "")
                    before <- z[1L]; after <- z[2L]
                }
            }
        }
        if(identical(q, "TeX")) {
            before <- "`"; after <- "'"
        }
        if(identical(q, "UTF-8")) {
            before <- "\u2018"; after <- "\u2019"
        }
        if(is.character(q) && length(q) >= 4L) {
            before <- q[1L]; after <- q[2L]
        }
    }
    paste0(before, x, after)
}"###,
    ),
    (
        "ngettext",
        r###"function(n, msg1, msg2, domain = NULL)
    .Internal(ngettext(n, msg1, msg2, domain))"###,
    ),
    ("l10n_info", r###"function() .Internal(l10n_info())"###),
];
