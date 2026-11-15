use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_arguments_closure",
            setup: "",
            source: "withRestarts(invokeRestart('r',function()17L),r=function(x)x())",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_arguments_closure.stdout"),
            stderr: include_str!("../raw/restart_arguments_closure.stderr"),
        },
        Row {
            id: "restart_arguments_environment",
            setup: "",
            source: "e<-new.env();e$x<-41L;withRestarts(invokeRestart('r',e),r=function(x)x$x)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_arguments_environment.stdout"),
            stderr: include_str!("../raw/restart_arguments_environment.stderr"),
        },
        Row {
            id: "restart_same_name_nearest",
            setup: "",
            source: "withRestarts(withRestarts(invokeRestart('r',21L),r=function(x)x+1L),r=function(x)0L)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_same_name_nearest.stdout"),
            stderr: include_str!("../raw/restart_same_name_nearest.stderr"),
        },
        Row {
            id: "restart_order",
            setup: "",
            source: "withRestarts(lapply(computeRestarts(),function(r)r[[1L]]),a=function()NULL,b=function()NULL)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_order.stdout"),
            stderr: include_str!("../raw/restart_order.stderr"),
        }
    ]
}
