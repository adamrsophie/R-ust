use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "warning_internal_null",
            setup: "",
            source: ".Internal(warning(FALSE,FALSE,FALSE,NULL))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_internal_null.stdout"),
            stderr: include_str!("../raw/warning_internal_null.stderr"),
        },
        Row {
            id: "warning_muffle_visibility",
            setup: "",
            source: "withCallingHandlers(warning('TEXT'),warning=function(w)invokeRestart('muffleWarning'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_muffle_visibility.stdout"),
            stderr: include_str!("../raw/warning_muffle_visibility.stderr"),
        },
        Row {
            id: "warning_error_cleanup",
            setup: "",
            source: "tryCatch(withCallingHandlers(warning('TEXT'),warning=function(w)stop('FAIL')),error=function(e)NULL);is.null(findRestart('muffleWarning'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_error_cleanup.stdout"),
            stderr: include_str!("../raw/warning_error_cleanup.stderr"),
        },
        Row {
            id: "warning_handler_output_order",
            setup: "",
            source: "withCallingHandlers({cat('BEFORE|');warning('TEXT');cat('AFTER|')},warning=function(w){cat('HANDLER|');invokeRestart('muffleWarning')})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/warning_handler_output_order.stdout"),
            stderr: include_str!("../raw/warning_handler_output_order.stderr"),
        }
    ]
}
