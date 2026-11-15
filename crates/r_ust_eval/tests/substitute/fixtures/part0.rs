use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_literal",
            setup: "",
            source: "substitute(1L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "substitute_global",
            setup: "x<-42L",
            source: "substitute(x)",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
        Row {
            id: "substitute_explicit_global",
            setup: "x<-42L",
            source: "substitute(x,globalenv())",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
        Row {
            id: "substitute_local",
            setup: "f<-function(x) substitute(x)",
            source: "f(42L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(42)]),
        },
    ]
}
