use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "dots_head_empty",
            setup: "f<-function(...)substitute(...(z))",
            source: "f()",
            stdout: "",
            expected: Sexp::pairlist(vec![ListElement {
                name: None,
                value: Sexp::Symbol("z".into()),
            }]),
        },
        Row {
            id: "dots_head_unbound",
            setup: "",
            source: "substitute(...(z),NULL)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("...".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Symbol("z".into()),
                }],
            },
        },
        Row {
            id: "dots_alone_empty",
            setup: "f<-function(...)substitute(...)",
            source: "f()",
            stdout: "",
            expected: Sexp::Nil,
        },
        Row {
            id: "dots_alone_many",
            setup: "f<-function(...)substitute(...)",
            source: "f(a=1L,b=2L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
    ]
}
