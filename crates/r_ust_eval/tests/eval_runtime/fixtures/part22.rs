use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "expression_error",
            setup: "",
            source:
                "eval(expression(cat('FIRST\\n'), stop('sequence failure'), cat('UNREACHED\\n')))",
            after: "",
            expected: None,
            visible: None,
            stdout: "FIRST\n",
            error: Some("sequence failure"),
            expected_type: None,
            class: &[],
            mode: None,
            length: None,
            print: "",
            warnings: &[],
            warning_calls: &[],
            warning_classes: &[],
        },
        Row {
            id: "expression_as_data",
            setup: "",
            source: "evalq(expression(x, x+1L))",
            after: "",
            expected: Some(Sexp::Expression {
                elements: vec![
                    ListElement {
                        name: None,
                        value: Sexp::Symbol("x".into()),
                    },
                    ListElement {
                        name: None,
                        value: Sexp::Call {
                            function: Box::new(Sexp::Symbol("+".into())),
                            arguments: vec![
                                CallArgument::positional(Sexp::Symbol("x".into())),
                                CallArgument::positional(Sexp::Integer(vec![Some(1)])),
                            ],
                        },
                    },
                ],
                names_present: false,
            }),
            visible: Some(true),
            stdout: "",
            error: None,
            expected_type: Some("expression"),
            class: &["expression"],
            mode: Some("expression"),
            length: Some(2),
            print: "expression(x, x + 1L)",
            warnings: &[],
            warning_calls: &[],
            warning_classes: &[],
        },
        Row {
            id: "break_outside",
            setup: "",
            source: "evalq(break)",
            after: "",
            expected: None,
            visible: None,
            stdout: "",
            error: Some("no loop for break/next, jumping to top level"),
            expected_type: None,
            class: &[],
            mode: None,
            length: None,
            print: "",
            warnings: &[],
            warning_calls: &[],
            warning_classes: &[],
        },
    ]
}
