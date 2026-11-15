use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "mixed",
            setup: "",
            source: "is.vector(x=expression(x),'list')",
            stdout: "",
            expected: Sexp::Logical(vec![Some(false)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "reordered",
            setup: "",
            source: "is.vector('list',x=expression(x))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(false)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "named_mode",
            setup: "",
            source: "is.vector(mode='expression',expression(x))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
    ]
}
