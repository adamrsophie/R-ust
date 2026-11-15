use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "frame",
            setup: "",
            source: "is.vector({cat(sys.nframe());expression(x)})",
            stdout: "0",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "visible",
            setup: "",
            source: "withVisible(is.vector(invisible(expression(x))))",
            stdout: "",
            expected: Sexp::List(vec![
                ListElement {
                    name: Some("value".into()),
                    value: Sexp::Logical(vec![Some(true)]),
                },
                ListElement {
                    name: Some("visible".into()),
                    value: Sexp::Logical(vec![Some(true)]),
                },
            ]),
            visible: true,
            warning: "",
        },
        Row {
            id: "source_direct_capture",
            setup: "",
            source: "expression('f'(x))",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: Sexp::Call {
                        function: Box::new(Sexp::Symbol("f".into())),
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
    ]
}
