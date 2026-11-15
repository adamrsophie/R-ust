use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_condition_filter",
            setup: "",
            source: "withRestarts(lapply(computeRestarts(simpleWarning('x')),function(r)r[[1L]]),a=list(handler=function()NULL,test=function(c)FALSE),b=list(handler=function()NULL,test=function(c)TRUE))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_condition_filter.stdout"),
            stderr: include_str!("../raw/restart_condition_filter.stderr"),
        },
        Row {
            id: "restart_missing_name",
            setup: "",
            source: "tryCatch(invokeRestart('absent'),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_missing_name.stdout"),
            stderr: include_str!("../raw/restart_missing_name.stderr"),
        },
        Row {
            id: "restart_try_missing",
            setup: "",
            source: "tryInvokeRestart('absent')",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_try_missing.stdout"),
            stderr: include_str!("../raw/restart_try_missing.stderr"),
        },
        Row {
            id: "restart_bad_spec",
            setup: "",
            source: "tryCatch(withRestarts(1L,r=2L),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_bad_spec.stdout"),
            stderr: include_str!("../raw/restart_bad_spec.stderr"),
        }
    ]
}
