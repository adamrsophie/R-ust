use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "dots_alone_missing",
            setup: "f<-function(...)substitute(...)",
            source: "f(a=,b=2L)",
            stdout: "",
            expected: Sexp::MissingValue,
        },
        Row {
            id: "missing_env",
            setup: "x<-1L",
            source: "substitute(x,env=)",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
        Row {
            id: "missing_named_expr_fill",
            setup: "",
            source: "substitute(expr=,x,list(x=3L))",
            stdout: "",
            expected: Sexp::Integer(vec![Some(3)]),
        },
        Row {
            id: "list_blank_name",
            setup: "y<-list(9L);names(y)<-\"\"",
            source: "substitute(x,y)",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
    ]
}
