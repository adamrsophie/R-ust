use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "expression_empty",
            setup: "",
            source: "expression()",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![],
                names_present: false,
            },
        },
        Row {
            id: "expression_one",
            setup: "",
            source: "expression(x)",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: Sexp::Symbol("x".into()),
                }],
                names_present: false,
            },
        },
        Row {
            id: "expression_nested",
            setup: "",
            source: "expression(x+1L, f(y))",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: None,
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("+".into())),
                            arguments: vec![
                                CallArgument {
                                    name: None,
                                    value: Sexp::Symbol("x".into()),
                                },
                                CallArgument {
                                    name: None,
                                    value: Sexp::Integer(vec![Some(1)]),
                                },
                            ],
                        },
                    },
                    ListElement {
                        name: None,
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("f".into())),
                            arguments: vec![CallArgument {
                                name: None,
                                value: Sexp::Symbol("y".into()),
                            }],
                        },
                    },
                ],
                names_present: false,
            },
        },
        Row {
            id: "expression_names",
            setup: "",
            source: "expression(a=x,y,b=3L)",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: Some("a".into()),
                        value: Sexp::Symbol("x".into()),
                    },
                    ListElement {
                        name: Some("".into()),
                        value: Sexp::Symbol("y".into()),
                    },
                    ListElement {
                        name: Some("b".into()),
                        value: Sexp::Integer(vec![Some(3)]),
                    },
                ],
                names_present: true,
            },
        },
    ]
}
