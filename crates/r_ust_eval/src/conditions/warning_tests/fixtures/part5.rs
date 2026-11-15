use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "warning_visibility_preserved",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "f<-function(x)invisible(x);withCallingHandlers(f(is.vector(m='expression',x=expression(x))),warning=function(w)invokeRestart('muffleWarning'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_visibility_preserved.stdout"),
            stderr: include_str!("../raw/warning_visibility_preserved.stderr"),
        },
        Row {
            id: "restart_arguments_named",
            setup: "",
            source: "withRestarts(invokeRestart('r',y=3L,x=2L),r=function(x,y)x*10L+y)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_arguments_named.stdout"),
            stderr: include_str!("../raw/restart_arguments_named.stderr"),
        },
        Row {
            id: "restart_arguments_defaults",
            setup: "",
            source: "withRestarts(invokeRestart('r'),r=function(x=7L,y=2L)x-y)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_arguments_defaults.stdout"),
            stderr: include_str!("../raw/restart_arguments_defaults.stderr"),
        },
        Row {
            id: "restart_arguments_quoted_call",
            setup: "",
            source: "withRestarts(invokeRestart('r',quote(stop('NOT_FORCED'))),r=function(x)identical(x,quote(stop('NOT_FORCED'))))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_arguments_quoted_call.stdout"),
            stderr: include_str!("../raw/restart_arguments_quoted_call.stderr"),
        }
    ]
}
