use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "replacement_call_not_recursive",
            setup: "",
            source: "substitute(x,list(x=quote(f(y)),y=1L,f=quote(g)))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("f".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("y".into()),
                }],
            },
        },
        Row {
            id: "dots_expansion_no_substitution",
            setup: "f<-function(...,x)substitute(g(...))",
            source: "f(y,x=7L)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("y".into()),
                }],
            },
        },
        Row {
            id: "dots_tag_is_replaced",
            setup: "f<-function(...)substitute(g(z=...))",
            source: "f(a=1L,b=2L)",
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
                        value: Sexp::Integer(vec![Some(2)]),
                    },
                ],
            },
        },
        Row {
            id: "dots_head_splice",
            setup: "f<-function(...)substitute(...(z))",
            source: "f(g,x=1L)",
            stdout: "",
            expected: Sexp::pairlist(vec![
                ListElement {
                    name: None,
                    value: Sexp::Symbol("g".into()),
                },
                ListElement {
                    name: Some("x".into()),
                    value: Sexp::Integer(vec![Some(1)]),
                },
                ListElement {
                    name: None,
                    value: Sexp::Symbol("z".into()),
                },
            ]),
        },
    ]
}
