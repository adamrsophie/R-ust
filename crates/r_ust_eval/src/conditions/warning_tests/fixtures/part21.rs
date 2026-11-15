use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "rebound_hook_error",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(as.integer('bad'),error=function(e)conditionMessage(e))",
            binding: ".signalSimpleWarning",
            replacement: "function(...){stop('HOOKFAIL')}",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/rebound_hook_error.stdout"),
            stderr: include_str!("../raw/rebound_hook_error.stderr"),
        },
        Row {
            id: "partial_complete_condition",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(is.vector(m='expression',x=expression(x)),warning=function(w)unclass(w))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_complete_condition.stdout"),
            stderr: include_str!("../raw/partial_complete_condition.stderr"),
        },
        Row {
            id: "partial_class",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(is.vector(m='expression',x=expression(x)),warning=function(w)class(w))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_class.stdout"),
            stderr: include_str!("../raw/partial_class.stderr"),
        },
        Row {
            id: "partial_hook_bypass",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w){cat(class(w)[[1L]]);invokeRestart(\"muffleWarning\")})",
            binding: ".signalSimpleWarning",
            replacement: "function(...)stop('SHOULD_BYPASS')",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_hook_bypass.stdout"),
            stderr: include_str!("../raw/partial_hook_bypass.stderr"),
        }
    ]
}
