use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "get_numeric_mode",
            setup: "",
            source:
                "x<-1L;list(get('x',mode='double'),get('x',mode='integer'),get('x',mode='numeric'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_numeric_mode.stdout"),
            stderr: include_str!("../raw/get_numeric_mode.stderr"),
        },
        Row {
            id: "get_symbol_name",
            setup: "",
            source: "x<-13L;get(as.name('x'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_symbol_name.stdout"),
            stderr: include_str!("../raw/get_symbol_name.stderr"),
        },
        Row {
            id: "get_pending_force_once",
            setup: "",
            source: "f<-function(x){get('x');get('x')};f({cat('ONCE|');7L})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_pending_force_once.stdout"),
            stderr: include_str!("../raw/get_pending_force_once.stderr"),
        },
        Row {
            id: "get_invalid_mode",
            setup: "",
            source: "tryCatch(get('x',mode='bad'),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_invalid_mode.stdout"),
            stderr: include_str!("../raw/get_invalid_mode.stderr"),
        },
    ]
}
