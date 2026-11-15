use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_list_env",
            setup: "",
            source: "substitute(x+y,list(x=1L,y=2L))",
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
            id: "substitute_pairlist_env",
            setup: "",
            source: "substitute(x+y,pairlist(x=1L,y=2L))",
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
            id: "substitute_duplicate_list",
            setup: "",
            source: "substitute(x,list(x=1L,x=2L))",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "substitute_unnamed_list",
            setup: "",
            source: "substitute(x,list(1L,x=2L))",
            stdout: "",
            expected: Sexp::Integer(vec![Some(2)]),
        },
    ]
}
