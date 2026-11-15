use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "literal",
            setup: "",
            source: "'is.vector'(x=expression(x),'list')",
            stdout: "",
            expected: Sexp::Logical(vec![Some(false)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "internal",
            setup: "",
            source: ".Internal(is.vector(expression(x),'expression'))",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
        Row {
            id: "force_order",
            setup: "",
            source: "is.vector(mode={cat('M');'any'},x={cat('X');expression(x)})",
            stdout: "XM",
            expected: Sexp::Logical(vec![Some(true)]),
            visible: true,
            warning: "",
        },
    ]
}
