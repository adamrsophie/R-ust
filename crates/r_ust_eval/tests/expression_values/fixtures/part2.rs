use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "expression_names_absent",
            setup: "",
            source: "is.null(names(expression(x,y)))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
        },
        Row {
            id: "expression_names_blank",
            setup: "",
            source: "names(expression(a=x,y))",
            stdout: "",
            expected: Sexp::String(vec![Some("a".into()), Some("".into())]),
        },
        Row {
            id: "expression_extract",
            setup: "",
            source: "expression(a=x+1L)[[1L]]",
            stdout: "",
            expected: Sexp::Call {
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
        Row {
            id: "expression_subset",
            setup: "",
            source: "expression(a=x,b=y)[2:1]",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: Some("b".into()),
                        value: Sexp::Symbol("y".into()),
                    },
                    ListElement {
                        name: Some("a".into()),
                        value: Sexp::Symbol("x".into()),
                    },
                ],
                names_present: true,
            },
        },
    ]
}
