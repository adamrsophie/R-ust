use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "partial_ignored_after_handler",
            setup: "options(warnPartialMatchArgs=TRUE);options(warn=-1)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)cat('HANDLED|'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_ignored_after_handler.stdout"),
            stderr: include_str!("../raw/partial_ignored_after_handler.stderr"),
        },
        Row {
            id: "partial_warn2_muffled",
            setup: "options(warnPartialMatchArgs=TRUE);options(warn=2)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)invokeRestart('muffleWarning'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_warn2_muffled.stdout"),
            stderr: include_str!("../raw/partial_warn2_muffled.stderr"),
        },
        Row {
            id: "partial_warn2_error",
            setup: "options(warnPartialMatchArgs=TRUE);options(warn=2)",
            source: "tryCatch(is.vector(m='expression',x=expression(x)),error=function(e)list(class(e),conditionMessage(e),conditionCall(e)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_warn2_error.stdout"),
            stderr: include_str!("../raw/partial_warn2_error.stderr"),
        },
        Row {
            id: "partial_exiting_warning",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(is.vector(m='expression',x=expression(x)),warning=function(w){cat('EXIT|');FALSE})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_exiting_warning.stdout"),
            stderr: include_str!("../raw/partial_exiting_warning.stderr"),
        }
    ]
}
