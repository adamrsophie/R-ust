use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "lapply_special_quote",
            setup: "",
            source: "lapply(list(1L),quote)",
            binding: "",
            replacement: "",
            error_message: "2 arguments passed to 'quote' which requires 1",
            status: 1,
            stdout: include_str!("../raw/lapply_special_quote.stdout"),
            stderr: include_str!("../raw/lapply_special_quote.stderr"),
        },
        Row {
            id: "lapply_primitive",
            setup: "",
            source: "lapply(list(1L,2L),is.integer)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_primitive.stdout"),
            stderr: include_str!("../raw/lapply_primitive.stderr"),
        },
        Row {
            id: "lapply_first_force_failure",
            setup: "",
            source: "x<-list(1L);f<-function(z){parent.frame()$X<-list();z};tryCatch(lapply(x,f),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_first_force_failure.stdout"),
            stderr: include_str!("../raw/lapply_first_force_failure.stderr"),
        },
        Row {
            id: "match_fun_descend_false",
            setup: "",
            source: "identity<-7L;tryCatch(match.fun('identity',descend=FALSE),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/match_fun_descend_false.stdout"),
            stderr: include_str!("../raw/match_fun_descend_false.stderr"),
        }
    ]
}
