use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "partial_warning_binding",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "is.vector(m='expression',x=expression(x))",
            binding: "warning",
            replacement: "function(...) {w<-list(...)[[1L]];cat(class(w)[[1L]]);cat(identical(parent.env(parent.frame()),.BaseNamespaceEnv));invisible(NULL)}",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_warning_binding.stdout"),
            stderr: include_str!("../raw/partial_warning_binding.stderr"),
        },
        Row {
            id: "warning_condition",
            setup: "",
            source: "withCallingHandlers(warning(simpleWarning('TEXT',quote(f()))),warning=function(w){cat(conditionMessage(w));invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_condition.stdout"),
            stderr: include_str!("../raw/warning_condition.stderr"),
        },
        Row {
            id: "warning_string",
            setup: "",
            source: "warning('TEXT')",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_string.stdout"),
            stderr: include_str!("../raw/warning_string.stderr"),
        },
        Row {
            id: "warning_caller_call",
            setup: "",
            source: "f<-function()withCallingHandlers(warning('TEXT'),warning=function(w){cat(identical(conditionCall(w),quote(f())));invokeRestart('muffleWarning')});f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_caller_call.stdout"),
            stderr: include_str!("../raw/warning_caller_call.stderr"),
        }
    ]
}
