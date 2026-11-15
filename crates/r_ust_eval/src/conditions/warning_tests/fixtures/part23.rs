use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "warning_call_false",
            setup: "",
            source: "warning('TEXT',call.=FALSE)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_call_false.stdout"),
            stderr: include_str!("../raw/warning_call_false.stderr"),
        },
        Row {
            id: "warning_immediate",
            setup: "options(warn=0)",
            source: "warning('TEXT',immediate.=TRUE)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_immediate.stdout"),
            stderr: include_str!("../raw/warning_immediate.stderr"),
        },
        Row {
            id: "warning_no_break",
            setup: "",
            source: "f<-function()warning('12345678901234567890123456789012345678901234567890123456789012345678901234567890',noBreaks.=TRUE);f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_no_break.stdout"),
            stderr: include_str!("../raw/warning_no_break.stderr"),
        },
        Row {
            id: "warning_internal_empty",
            setup: "",
            source: ".Internal(warning(FALSE,FALSE,FALSE,character()))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_internal_empty.stdout"),
            stderr: include_str!("../raw/warning_internal_empty.stderr"),
        }
    ]
}
