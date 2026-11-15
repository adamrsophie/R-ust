use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_nonforce",
            setup: "f<-function(x) substitute(x)",
            source: "f(stop(\"BAD\"))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("stop".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::String(vec![Some("BAD".into())]),
                }],
            },
        },
        Row {
            id: "substitute_forced_promise",
            setup: "f<-function(x){force(x);substitute(x)}",
            source: "f(1L+2L)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("+".into())),
                arguments: vec![
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(1)]),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(2)]),
                    },
                ],
            },
        },
        Row {
            id: "substitute_default",
            setup: "f<-function(x=y+1L) substitute(x)",
            source: "f()",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("+".into())),
                arguments: vec![
                    CallArgument {
                        name: None,
                        value: Sexp::Symbol("y".into()),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(1)]),
                    },
                ],
            },
        },
        Row {
            id: "substitute_forwarded",
            setup: "g<-function(y) substitute(y);f<-function(x) g(x)",
            source: "f(stop(\"BAD\"))",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
    ]
}
