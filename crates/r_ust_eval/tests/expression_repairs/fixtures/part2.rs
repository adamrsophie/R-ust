use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "named_empty_default",
            setup: "",
            source: "is.vector(expression(x),mode=)",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "alias",
            setup: "",
            source: "f<-is.vector;f(x=expression(x),'list')",
            stdout: "",
            expected: Sexp::Logical(vec![Some(false)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "computed",
            setup: "",
            source: "(identity(is.vector))(x=expression(x),'list')",
            stdout: "",
            expected: Sexp::Logical(vec![Some(false)]),
            visible: true,
            warning: "",
        },
    ]
}
