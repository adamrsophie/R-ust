use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "expression_hole_identity",
            setup: "",
            source: "identical(expression(,)[[1L]],quote(expr=))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
        },
        Row {
            id: "expression_nested_data",
            setup: "",
            source: "list(expression(x),expression())",
            stdout: "",
            expected: Sexp::List(vec![
                ListElement {
                    name: None,
                    value: Sexp::Expression {
                        elements: vec![ListElement {
                            name: None,
                            value: Sexp::Symbol("x".into()),
                        }],
                        names_present: false,
                    },
                },
                ListElement {
                    name: None,
                    value: Sexp::Expression {
                        elements: vec![],
                        names_present: false,
                    },
                },
            ]),
        },
        Row {
            id: "expression_nonforce",
            setup: "",
            source: "expression(stop(\"BAD\"),{cat(\"BAD\");1L})",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: None,
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("stop".into())),
                            arguments: vec![CallArgument {
                                name: None,
                                value: Sexp::String(vec![Some("BAD".into())]),
                            }],
                        },
                    },
                    ListElement {
                        name: None,
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("{".into())),
                            arguments: vec![
                                CallArgument {
                                    name: None,
                                    value: Sexp::Call {
                                        function: Box::new(Sexp::Symbol("cat".into())),
                                        arguments: vec![CallArgument {
                                            name: None,
                                            value: Sexp::String(vec![Some("BAD".into())]),
                                        }],
                                    },
                                },
                                CallArgument {
                                    name: None,
                                    value: Sexp::Integer(vec![Some(1)]),
                                },
                            ],
                        },
                    },
                ],
                names_present: false,
            },
        },
        Row {
            id: "expression_self_data",
            setup: "",
            source: "e<-expression(stop(\"BAD\")); identical(e,expression(stop(\"BAD\")))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
        },
    ]
}
