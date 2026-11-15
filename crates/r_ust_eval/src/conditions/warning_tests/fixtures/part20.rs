use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "as_environment_parent",
            setup: "",
            source: "f<-function()identical(as.environment(-1L),parent.frame());f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_environment_parent.stdout"),
            stderr: include_str!("../raw/as_environment_parent.stderr"),
        },
        Row {
            id: "as_environment_top_missing_parent",
            setup: "",
            source: "as.environment(-1L)",
            binding: "",
            replacement: "",
            error_message: "no enclosing environment",
            status: 1,
            stdout: include_str!("../raw/as_environment_top_missing_parent.stdout"),
            stderr: include_str!("../raw/as_environment_top_missing_parent.stderr"),
        },
        Row {
            id: "rebound_hook_source_call",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "as.integer('bad');observed",
            binding: ".signalSimpleWarning",
            replacement: "function(msg,call){observed<<-list(msg,call,identical(parent.frame(),baseenv()));invisible(NULL)}",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/rebound_hook_source_call.stdout"),
            stderr: include_str!("../raw/rebound_hook_source_call.stderr"),
        },
        Row {
            id: "rebound_hook_argument_order",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "as.integer('bad');observed",
            binding: ".signalSimpleWarning",
            replacement: "function(call,msg){observed<<-list(call,msg);invisible(NULL)}",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/rebound_hook_argument_order.stdout"),
            stderr: include_str!("../raw/rebound_hook_argument_order.stderr"),
        }
    ]
}
