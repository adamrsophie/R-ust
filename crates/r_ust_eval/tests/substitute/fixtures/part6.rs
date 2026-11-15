use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_dots_empty",
            setup: "f<-function(...) substitute(g(...))",
            source: "f()",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![],
            },
        },
        Row {
            id: "substitute_dots_holes",
            setup: "f<-function(...) substitute(g(...))",
            source: "f(a=,b=2L,)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![
                    CallArgument {
                        name: Some("a".into()),
                        value: Sexp::MissingValue,
                    },
                    CallArgument {
                        name: Some("b".into()),
                        value: Sexp::Integer(vec![Some(2)]),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::MissingValue,
                    },
                ],
            },
        },
        Row {
            id: "substitute_dots_unbound",
            setup: "",
            source: "substitute(f(...),list())",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("...".into()),
                }],
            },
        },
        Row {
            id: "substitute_dots_null",
            setup: "",
            source: "substitute(f(...),list(...=NULL))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![],
            },
        },
    ]
}
