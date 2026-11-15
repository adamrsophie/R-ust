use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_cleanup_return",
            setup: "",
            source: "f<-function(){withRestarts(return(23L),r=function()0L);99L};list(f(),is.null(findRestart('r')))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_return.stdout"),
            stderr: include_str!("../raw/restart_cleanup_return.stderr"),
        },
        Row {
            id: "restart_cleanup_break_next",
            setup: "",
            source: "out<-0L;for(i in 1:3){withRestarts({if(i==2L)next;if(i==3L)break;out<-out+i},r=function()0L)};list(out,is.null(findRestart('r')))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_break_next.stdout"),
            stderr: include_str!("../raw/restart_cleanup_break_next.stderr"),
        },
        Row {
            id: "restart_cleanup_onexit_order",
            setup: "",
            source: "f<-function(){on.exit(cat('EXIT|'));invokeRestart('r',8L)};withRestarts(f(),r=function(x){cat('HANDLER|');x})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_onexit_order.stdout"),
            stderr: include_str!("../raw/restart_cleanup_onexit_order.stderr"),
        },
        Row {
            id: "restart_cleanup_onexit_sees_stack",
            setup: "",
            source: "f<-function(){on.exit(cat(is.null(findRestart('r'))));withRestarts(7L,r=function()0L)};f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_cleanup_onexit_sees_stack.stdout"),
            stderr: include_str!("../raw/restart_cleanup_onexit_sees_stack.stderr"),
        }
    ]
}
