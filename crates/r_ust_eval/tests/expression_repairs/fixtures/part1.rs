use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "partial",
            setup: "",
            source: "is.vector(m='expression',x=expression(x))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "default",
            setup: "",
            source: "is.vector(expression(x))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "empty_default",
            setup: "",
            source: "is.vector(expression(x),)",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
    ]
}
