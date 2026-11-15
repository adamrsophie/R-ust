use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "partial_condition_shape",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch(is.vector(m='expression',x=expression(x)),warning=function(w)list(class(w),conditionMessage(w),conditionCall(w)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_condition_shape.stdout"),
            stderr: include_str!("../raw/partial_condition_shape.stderr"),
        },
        Row {
            id: "partial_alias_actual_call",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "a<-is.vector;tryCatch(a(m='expression',x=expression(x)),warning=function(w)conditionCall(w))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_alias_actual_call.stdout"),
            stderr: include_str!("../raw/partial_alias_actual_call.stderr"),
        },
        Row {
            id: "partial_computed_actual_call",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "tryCatch((is.vector)(m='expression',x=expression(x)),warning=function(w)conditionCall(w))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/partial_computed_actual_call.stdout"),
            stderr: include_str!("../raw/partial_computed_actual_call.stderr"),
        },
        Row {
            id: "handler_policy_change",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)options(warn=-1))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/handler_policy_change.stdout"),
            stderr: include_str!("../raw/handler_policy_change.stderr"),
        }
    ]
}
