use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "do_call_direct_return",
            setup: "",
            source: "f<-function(){do.call('return',list(19L),envir=environment());99L};f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_direct_return.stdout"),
            stderr: include_str!("../raw/do_call_direct_return.stderr"),
        },
        Row {
            id: "do_call_empty_args",
            setup: "",
            source: "do.call(function()12L,list())",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_empty_args.stdout"),
            stderr: include_str!("../raw/do_call_empty_args.stderr"),
        },
        Row {
            id: "do_call_invalid_what",
            setup: "",
            source:
                "tryCatch(do.call(quote(identity),list(1L)),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_invalid_what.stdout"),
            stderr: include_str!("../raw/do_call_invalid_what.stderr"),
        },
        Row {
            id: "do_call_invalid_args",
            setup: "",
            source: "tryCatch(do.call(identity,1L),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_invalid_args.stdout"),
            stderr: include_str!("../raw/do_call_invalid_args.stderr"),
        },
    ]
}
