use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "pairlist_duplicate",
            setup: "",
            source: "substitute(x,pairlist(x=1L,x=2L))",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "explicit_env_once",
            setup: "n<-0L",
            source: "list(substitute(x,{n<-n+1L;list(x=n)}),n)",
            stdout: "",
            expected: Sexp::List(vec![
                ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                },
                ListElement {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                },
            ]),
        },
        Row {
            id: "alias_nonforce",
            setup: "s<-substitute",
            source: "s(stop(\"BAD\"))",
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
            id: "computed_nonforce",
            setup: "",
            source: "(if(TRUE)substitute else stop)(stop(\"BAD\"))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("stop".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::String(vec![Some("BAD".into())]),
                }],
            },
        },
    ]
}
