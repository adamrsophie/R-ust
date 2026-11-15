use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "env_next_control",
            setup: "n<-0L",
            source: "{for(i in 1L:2L){substitute(x,next);n<-99L};n}",
            stdout: "",
            expected: Sexp::Integer(vec![Some(0)]),
        },
        Row {
            id: "source_quote_string_head",
            setup: "",
            source: "quote(\"f\"(x))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("x".into()),
                }],
            },
        },
        Row {
            id: "source_quote_alias_string_head",
            setup: "q<-quote",
            source: "q(\"f\"(x))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("x".into()),
                }],
            },
        },
        Row {
            id: "source_substitute_string_head",
            setup: "",
            source: "substitute(\"f\"(x),list(f=quote(g),x=1L))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                }],
            },
        },
    ]
}
