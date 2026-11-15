use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "coercion_muffled",
            setup: "",
            source: "withCallingHandlers(as.integer('bad'),warning=function(w){cat(conditionMessage(w));invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/coercion_muffled.stdout"),
            stderr: include_str!("../raw/coercion_muffled.stderr"),
        },
        Row {
            id: "coercion_caller_call",
            setup: "",
            source: "f<-function()withCallingHandlers(as.integer('bad'),warning=function(w){cat(identical(conditionCall(w),quote(f())));invokeRestart('muffleWarning')});f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/coercion_caller_call.stdout"),
            stderr: include_str!("../raw/coercion_caller_call.stderr"),
        },
        Row {
            id: "simple_warning_only_signals",
            setup: "",
            source: "withCallingHandlers(signalCondition(simpleWarning('ONLY')),warning=function(w){cat(conditionMessage(w));tryInvokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/simple_warning_only_signals.stdout"),
            stderr: include_str!("../raw/simple_warning_only_signals.stderr"),
        },
        Row {
            id: "suppress_warnings_real_helper",
            setup: "options(warnPartialMatchArgs=TRUE)",
            source: "suppressWarnings(is.vector(m='expression',x=expression(x)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/suppress_warnings_real_helper.stdout"),
            stderr: include_str!("../raw/suppress_warnings_real_helper.stderr"),
        }
    ]
}
