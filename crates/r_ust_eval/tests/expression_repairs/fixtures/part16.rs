use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "source_grouped_capture",
            setup: "",
            source: "expression(('f')(x))",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: Sexp::Call {
                        function: Box::new(Sexp::Call {
                            function: Box::new(Sexp::Symbol("(".into())),
                            arguments: vec![CallArgument {
                                name: None,
                                value: Sexp::String(vec![Some("f".into())]),
                            }],
                        }),
                        arguments: vec![CallArgument {
                            name: None,
                            value: Sexp::Symbol("x".into()),
                        }],
                    },
                }],
                names_present: false,
            },
            visible: true,
            warning: "",
        },
        Row {
            id: "source_computed_capture",
            setup: "",
            source: "expression((identity('f'))(x))",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: Sexp::Call {
                        function: Box::new(Sexp::Call {
                            function: Box::new(Sexp::Symbol("(".into())),
                            arguments: vec![CallArgument {
                                name: None,
                                value: Sexp::Call {
                                    function: Box::new(Sexp::Symbol("identity".into())),
                                    arguments: vec![CallArgument {
                                        name: None,
                                        value: Sexp::String(vec![Some("f".into())]),
                                    }],
                                },
                            }],
                        }),
                        arguments: vec![CallArgument {
                            name: None,
                            value: Sexp::Symbol("x".into()),
                        }],
                    },
                }],
                names_present: false,
            },
            visible: true,
            warning: "",
        },
        Row {
            id: "source_direct_execute",
            setup: "",
            source: "f<-function()1L;'f'()",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
            visible: true,
            warning: "",
        },
    ]
}
