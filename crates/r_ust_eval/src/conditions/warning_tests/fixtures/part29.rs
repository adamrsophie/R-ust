use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "match_fun_invalid",
            setup: "",
            source: "tryCatch(match.fun(7L),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/match_fun_invalid.stdout"),
            stderr: include_str!("../raw/match_fun_invalid.stderr"),
        },
        Row {
            id: "asenv_local_frame",
            setup: "",
            source: "f<-function(){e<-as.environment(-1L);identical(e,globalenv())};f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/asenv_local_frame.stdout"),
            stderr: include_str!("../raw/asenv_local_frame.stderr"),
        },
    ]
}
