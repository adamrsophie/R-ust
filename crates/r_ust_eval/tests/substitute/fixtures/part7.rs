use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_missing_expr",
            setup: "",
            source: "substitute()",
            stdout: "",
            expected: Sexp::MissingValue,
        },
        Row {
            id: "substitute_dots_alone",
            setup: "",
            source: "(function(...) substitute(...))(1L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "substitute_named_expr_positional_env",
            setup: "x<-list(y=9L)",
            source: "substitute(x,expr=y)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(9)]),
        },
        Row {
            id: "replacement_not_recursive",
            setup: "",
            source: "substitute(x,list(x=quote(y),y=1L))",
            stdout: "",
            expected: Sexp::Symbol("y".into()),
        },
    ]
}
