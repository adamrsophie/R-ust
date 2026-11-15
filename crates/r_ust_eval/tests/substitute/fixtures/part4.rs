use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_head",
            setup: "",
            source: "substitute(f(x),list(f=quote(g),x=1L))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("g".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                }],
            },
        },
        Row {
            id: "substitute_env_leaf",
            setup: "e<-new.env()",
            source: "identical(substitute(x,list(x=e)),e)",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
        },
        Row {
            id: "substitute_closure_leaf",
            setup: "f<-function() 1L",
            source: "identical(substitute(f,list(f=f)),f)",
            stdout: "",
            expected: Sexp::Logical(vec![Some(true)]),
        },
        Row {
            id: "substitute_expression_opaque",
            setup: "",
            source: "substitute(expression(x),list(x=1L))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("expression".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Integer(vec![Some(1)]),
                }],
            },
        },
    ]
}
