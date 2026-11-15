use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "do_call_illegal_internal",
            setup: "",
            source: "tryCatch(do.call('.Internal',list()),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_illegal_internal.stdout"),
            stderr: include_str!("../raw/do_call_illegal_internal.stderr"),
        },
        Row {
            id: "get_default_caller",
            setup: "",
            source: "x<-1L;f<-function(){x<-7L;get('x')};f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_default_caller.stdout"),
            stderr: include_str!("../raw/get_default_caller.stderr"),
        },
        Row {
            id: "get_function_skips_nonfunction",
            setup: "",
            source: "f<-function()11L;g<-function(){f<-7L;get('f',mode='function')()};g()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_function_skips_nonfunction.stdout"),
            stderr: include_str!("../raw/get_function_skips_nonfunction.stderr"),
        },
        Row {
            id: "get_inherits_false",
            setup: "",
            source: "x<-1L;g<-function()tryCatch(get('x',inherits=FALSE),error=function(e)conditionMessage(e));g()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_inherits_false.stdout"),
            stderr: include_str!("../raw/get_inherits_false.stderr"),
        }
    ]
}
