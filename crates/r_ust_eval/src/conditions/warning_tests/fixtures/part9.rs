use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_stale",
            setup: "",
            source: "saved<-withRestarts(findRestart('r'),r=function()1L);tryCatch(invokeRestart(saved),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_stale.stdout"),
            stderr: include_str!("../raw/restart_stale.stderr"),
        },
        Row {
            id: "restart_stale_during_other",
            setup: "",
            source: "saved<-withRestarts(findRestart('r'),r=function()1L);withRestarts(tryCatch(invokeRestart(saved),error=function(e)conditionMessage(e)),s=function()2L)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_stale_during_other.stdout"),
            stderr: include_str!("../raw/restart_stale_during_other.stderr"),
        },
        Row {
            id: "restart_cleanup_value",
            setup: "",
            source: "withRestarts(7L,r=function()0L);is.null(findRestart('r'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_value.stdout"),
            stderr: include_str!("../raw/restart_cleanup_value.stderr"),
        },
        Row {
            id: "restart_cleanup_error",
            setup: "",
            source: "tryCatch(withRestarts(stop('FAIL'),r=function()0L),error=function(e)NULL);is.null(findRestart('r'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_error.stdout"),
            stderr: include_str!("../raw/restart_cleanup_error.stderr"),
        }
    ]
}
