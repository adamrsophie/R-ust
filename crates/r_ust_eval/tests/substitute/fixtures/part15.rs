use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "explicit_env_no_extra_activation",
            setup: "f<-function(){n<-0L;substitute(x,{n<-sys.nframe();environment()});n}",
            source: "f()",
            stdout: "",
            expected: Sexp::Integer(vec![Some(1)]),
        },
        Row {
            id: "invalid_env_condition_call",
            setup: "",
            source: "tryCatch(substitute(x,1L),error=function(e)conditionCall(e))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("substitute".into())),
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
            id: "alias_invalid_env_condition_call",
            setup: "s<-substitute;f<-function()tryCatch(s(x,1L),error=function(e)conditionCall(e))",
            source: "f()",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("s".into())),
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
            id: "unused_condition_call",
            setup: "",
            source: "tryCatch(substitute(x,NULL,z=1L),error=function(e)conditionCall(e))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("substitute".into())),
                arguments: vec![
                    CallArgument {
                        name: None,
                        value: Sexp::Symbol("x".into()),
                    },
                    CallArgument {
                        name: None,
                        value: Sexp::Nil,
                    },
                    CallArgument {
                        name: Some("z".into()),
                        value: Sexp::Integer(vec![Some(1)]),
                    },
                ],
            },
        },
    ]
}
