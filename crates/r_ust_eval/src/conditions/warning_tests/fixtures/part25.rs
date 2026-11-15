use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "lapply_language_not_re_evaluated",
            setup: "",
            source: "lapply(list(quote(x),quote(stop('NO')),function()1L,new.env()),enquote)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_language_not_re_evaluated.stdout"),
            stderr: include_str!("../raw/lapply_language_not_re_evaluated.stderr"),
        },
        Row {
            id: "lapply_expression_names",
            setup: "",
            source: "lapply(expression(a=x,y=1L),identity)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_expression_names.stdout"),
            stderr: include_str!("../raw/lapply_expression_names.stderr"),
        },
        Row {
            id: "lapply_names_empty_present",
            setup: "",
            source:
                "x<-list();names(x)<-character();list(lapply(x,identity),names(lapply(x,identity)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_names_empty_present.stdout"),
            stderr: include_str!("../raw/lapply_names_empty_present.stderr"),
        },
        Row {
            id: "lapply_names_na",
            setup: "",
            source:
                "x<-list(1L,2L);names(x)<-c(NA_character_,NA_character_);names(lapply(x,identity))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_names_na.stdout"),
            stderr: include_str!("../raw/lapply_names_na.stderr"),
        },
    ]
}
