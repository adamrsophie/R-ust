use super::*;
pub(super) fn value() -> Sexp {
    Sexp::pairlist(vec![
        ListElement {
            name: Some("expr".into()),
            value: Sexp::MissingValue,
        },
        ListElement {
            name: Some("envir".into()),
            value: Sexp::Call {
                function: Box::new(Sexp::Symbol("parent.frame".into())),
                arguments: vec![],
            },
        },
        ListElement {
            name: Some("enclos".into()),
            value: Sexp::Call {
                function: Box::new(Sexp::Symbol("if".into())),
                arguments: vec![
                    CallArgument::positional(Sexp::Call {
                        function: Box::new(Sexp::Symbol("||".into())),
                        arguments: vec![
                            CallArgument::positional(Sexp::Call {
                                function: Box::new(Sexp::Symbol("is.list".into())),
                                arguments: vec![CallArgument::positional(Sexp::Symbol(
                                    "envir".into(),
                                ))],
                            }),
                            CallArgument::positional(Sexp::Call {
                                function: Box::new(Sexp::Symbol("is.pairlist".into())),
                                arguments: vec![CallArgument::positional(Sexp::Symbol(
                                    "envir".into(),
                                ))],
                            }),
                        ],
                    }),
                    CallArgument::positional(Sexp::Call {
                        function: Box::new(Sexp::Symbol("parent.frame".into())),
                        arguments: vec![],
                    }),
                    CallArgument::positional(Sexp::Call {
                        function: Box::new(Sexp::Symbol("baseenv".into())),
                        arguments: vec![],
                    }),
                ],
            },
        },
    ])
}
