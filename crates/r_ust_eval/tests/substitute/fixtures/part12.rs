use super::*;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "primitive_nonforce",
            setup: "",
            source: ".Primitive(\"substitute\")(stop(\"BAD\"))",
            stdout: "",
            expected: Sexp::Call {
                function: Box::new(Sexp::Symbol("stop".into())),
                arguments: vec![CallArgument {
                    name: None,
                    value: Sexp::String(vec![Some("BAD".into())]),
                }],
            },
        },
        Row {
            id: "caller_frame",
            setup: "f<-function(x){s<-substitute;s(x)}",
            source: "f(7L)",
            stdout: "",
            expected: Sexp::Integer(vec![Some(7)]),
        },
        Row {
            id: "env_return_control",
            setup: "f<-function(){substitute(x,return(7L));stop(\"BAD\")}",
            source: "f()",
            stdout: "",
            expected: Sexp::Integer(vec![Some(7)]),
        },
        Row {
            id: "env_break_control",
            setup: "n<-0L",
            source: "{for(i in 1L:2L){substitute(x,break);n<-99L};n}",
            stdout: "",
            expected: Sexp::Integer(vec![Some(0)]),
        },
    ]
}
