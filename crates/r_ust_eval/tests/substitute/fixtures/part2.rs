use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "substitute_local_binding",
            setup: "f<-function(){x<-7L;substitute(x)}",
            source: "f()",
            stdout: "",
            expected: Sexp::Integer(vec![Some(7)]),
        },
        Row {
            id: "substitute_no_parent",
            setup: "p<-new.env();p$x<-7L;e<-new.env(parent=p)",
            source: "substitute(x,e)",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
        Row {
            id: "substitute_explicit_env",
            setup: "e<-new.env();e$x<-7L",
            source: "substitute(x+z,e)",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("+".into())),
                arguments: vec![
                    CallArgument {
                        name: None,
                        value: Sexp::Integer(vec![Some(7)]),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::Symbol("z".into()),
                    },
                ],
            },
        },
        Row {
            id: "substitute_null_env",
            setup: "x<-7L",
            source: "substitute(x,NULL)",
            stdout: "",
            expected: Sexp::Symbol("x".into()),
        },
    ]
}
