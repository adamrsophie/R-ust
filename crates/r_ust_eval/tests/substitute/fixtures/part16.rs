use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "quote_control_expression_nonforce",
            setup: "q<-quote",
            source: "q(return(7L))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("return".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::Integer(vec![Some(7)]),
                }],
            },
        },
        Row {
            id: "dots_splice_disables_lookup",
            setup: "f<-function(...,x){g<-quote(h);substitute(g(...))}",
            source: "f(a=x,x=7L)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("h".into())),
                arguments: vec![CallArgument {
                    name: Some("a".into()),
                    value: Sexp::Symbol("x".into()),
                }],
            },
        },
    ]
}
