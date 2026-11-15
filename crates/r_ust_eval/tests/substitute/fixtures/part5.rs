use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_explicit_env_order",
            setup: "",
            source: "substitute(stop(\"BAD\"),{cat(\"ENV\\n\");list()})",
            stdout: "ENV\n",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("stop".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::String(vec![Some("BAD".into())]),
                }],
            },
        },
        Row {
            id: "substitute_match_reorder",
            setup: "",
            source: "substitute(env=list(x=3L),expr=x)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(3)]),
        },
        Row {
            id: "substitute_partial_match",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "substitute(ex=x,en=list(x=3L))",
            stdout: "",
            expected: Sexp::Integer(vec![Some(3)]),
        },
        Row {
            id: "substitute_dots_named",
            setup: "f<-function(...) substitute(g(...))",
            source: "f(a=1L,b=stop(\"BAD\"))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![
                    CallArgument {
                        name: Some("a".into()),
                        value: Sexp::Integer(vec![Some(1)]),
                    },
                    CallArgument {
                        name: Some("b".into()),
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("stop".into())),
                            arguments: vec![CallArgument {
                                name: None,
                                value: Sexp::String(vec![Some("BAD".into())]),
                            }],
                        },
                    },
                ],
            },
        },
    ]
}
