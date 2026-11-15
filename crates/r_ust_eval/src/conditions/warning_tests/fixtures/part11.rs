use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_visibility",
            setup: "",
            source: "withRestarts(invokeRestart('r'),r=function()invisible(7L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_visibility.stdout"),
            stderr: include_str!("../raw/restart_visibility.stderr"),
        },
        Row {
            id: "restart_handler_parent",
            setup: "",
            source: "withRestarts(invokeRestart('r'),r=function()list(is.environment(parent.frame()),is.null(findRestart('r'))))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_handler_parent.stdout"),
            stderr: include_str!("../raw/restart_handler_parent.stderr"),
        },
        Row {
            id: "restart_internal_bad",
            setup: "",
            source: "tryCatch(.Internal(.addRestart(1L)),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_internal_bad.stdout"),
            stderr: include_str!("../raw/restart_internal_bad.stderr"),
        },
        Row {
            id: "restart_internal_index",
            setup: "",
            source: "list(.Internal(.getRestart(1L))[[1L]],is.null(.Internal(.getRestart(2L))),is.null(.Internal(.getRestart(0L))))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_internal_index.stdout"),
            stderr: include_str!("../raw/restart_internal_index.stderr"),
        }
    ]
}
