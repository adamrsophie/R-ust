use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "expression_named_empty",
            setup: "",
            source: "e<-expression(); names(e)<-character(); attributes(e)",
            stdout: "",
            expected: Sexp::List(vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![]),
            }]),
        },
        Row {
            id: "expression_symbol_self_evaluation",
            setup: "e<-expression(stop(\"BAD\"))",
            source: "e",
            stdout: "",
            expected: Sexp::Expression {
                elements: vec![ListElement {
                    name: None,
                    value: Sexp::Call {
                        function: Box::new(Sexp::Symbol("stop".into())),
                        arguments: vec![CallArgument {
                            name: None,
                            value: Sexp::String(vec![Some("BAD".into())]),
                        }],
                    },
                }],
                names_present: false,
            },
        },
        Row {
            id: "expression_print_names_holes",
            setup: "",
            source: "print(expression(a=x,,y)); expression(a=x,,y)",
            stdout: "expression(a = x, , y)\n",
            expected: Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: Some("a".into()),
                        value: Sexp::Symbol("x".into()),
                    },
                    ListElement {
                        name: Some("".into()),
                        value: Sexp::MissingValue,
                    },
                    ListElement {
                        name: Some("".into()),
                        value: Sexp::Symbol("y".into()),
                    },
                ],
                names_present: true,
            },
        },
    ]
}
