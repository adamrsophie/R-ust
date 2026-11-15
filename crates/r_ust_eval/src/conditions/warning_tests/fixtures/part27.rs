use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "lapply_dots_order",
            setup: "",
            source: "lapply(list(1L,2L),function(x,y){cat(x);y},y={cat('DOT|');9L})",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_dots_order.stdout"),
            stderr: include_str!("../raw/lapply_dots_order.stderr"),
        },
        Row {
            id: "lapply_character_function",
            setup: "",
            source: "lapply(list(1L,2L),'identity')",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_character_function.stdout"),
            stderr: include_str!("../raw/lapply_character_function.stderr"),
        },
        Row {
            id: "lapply_symbol_function",
            setup: "",
            source: "lapply(list(1L,2L),as.name('identity'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_symbol_function.stdout"),
            stderr: include_str!("../raw/lapply_symbol_function.stderr"),
        },
        Row {
            id: "lapply_nonfunction_recovery",
            setup: "",
            source: "identity<-17L;lapply(list(1L,2L),identity)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_nonfunction_recovery.stdout"),
            stderr: include_str!("../raw/lapply_nonfunction_recovery.stderr"),
        },
    ]
}
