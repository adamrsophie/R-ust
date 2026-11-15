use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "required_partial_muffle",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w){cat(conditionMessage(w));invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/required_partial_muffle.stdout"),
            stderr: include_str!("../raw/required_partial_muffle.stderr"),
        },
        Row {
            id: "partial_unmuffled",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "is.vector(m='expression',x=expression(x))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_unmuffled.stdout"),
            stderr: include_str!("../raw/partial_unmuffled.stderr"),
        },
        Row {
            id: "partial_deferred",
            setup: "options(warnPartialMatchArgs=TRUE);options(warn=0)",
            source: "is.vector(m='expression',x=expression(x))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_deferred.stdout"),
            stderr: include_str!("../raw/partial_deferred.stderr"),
        },
        Row {
            id: "partial_calling_unmuffled",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)cat('HANDLED|'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_calling_unmuffled.stdout"),
            stderr: include_str!("../raw/partial_calling_unmuffled.stderr"),
        }
    ]
}
