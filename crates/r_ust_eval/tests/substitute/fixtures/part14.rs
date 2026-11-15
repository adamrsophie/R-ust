use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "source_substitute_alias_string_head",
            setup: "s<-substitute",
            source: "s(\"f\"(x),list(f=quote(g),x=1L))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                }],
            },
        },
        Row {
            id: "source_string_head_execution",
            setup: "f<-function(x)x",
            source: "\"f\"(1L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "mixed_blank_and_na_list_names",
            setup: "e<-list(9L,2L);names(e)<-c(NA_character_,\"x\")",
            source: "substitute(f(`NA`,x),e)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(9)]),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(2)]),
                    },
                ],
            },
        },
        Row {
            id: "mixed_unnamed_list_does_not_bind_na",
            setup: "",
            source: "substitute(`NA`,list(9L,x=2L))",
            stdout: "",
            expected: Sexp::Symbol("NA".into()),
        },
    ]
}
