use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "handler_raises_error",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)stop('HANDLER')),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/handler_raises_error.stdout"),
            stderr: include_str!("../raw/handler_raises_error.stderr"),
        },
        Row {
            id: "handler_nested_warning",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w){cat('INNER|');is.vector(m='expression',x=expression(x));cat('RESUME|');invokeRestart('muffleWarning')}),warning=function(w){cat('OUTER|');invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/handler_nested_warning.stdout"),
            stderr: include_str!("../raw/handler_nested_warning.stderr"),
        },
        Row {
            id: "handler_real_parent",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w){cat(identical(parent.frame(),globalenv()));invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/handler_real_parent.stdout"),
            stderr: include_str!("../raw/handler_real_parent.stderr"),
        },
        Row {
            id: "handler_return_only_returns_handler",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "f<-function(){withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)return(9L));7L};f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/handler_return_only_returns_handler.stdout"),
            stderr: include_str!("../raw/handler_return_only_returns_handler.stderr"),
        }
    ]
}
