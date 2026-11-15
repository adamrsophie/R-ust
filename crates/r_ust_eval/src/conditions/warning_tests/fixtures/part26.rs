use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "lapply_actual_call",
            setup: "",
            source: "lapply(list(1L,2L),function(x)sys.call())",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_actual_call.stdout"),
            stderr: include_str!("../raw/lapply_actual_call.stderr"),
        },
        Row {
            id: "lapply_source_expression",
            setup: "",
            source: "lapply(list(1L,2L),function(x)substitute(x))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_source_expression.stdout"),
            stderr: include_str!("../raw/lapply_source_expression.stderr"),
        },
        Row {
            id: "lapply_parent_frame",
            setup: "",
            source: "lapply(list(1L,2L),function(x)list(identical(sys.function(sys.parent()),lapply),get('i',parent.frame())))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_parent_frame.stdout"),
            stderr: include_str!("../raw/lapply_parent_frame.stderr"),
        },
        Row {
            id: "lapply_first_before_unused",
            setup: "",
            source: "lapply(list(1L),function(x)7L)",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/lapply_first_before_unused.stdout"),
            stderr: include_str!("../raw/lapply_first_before_unused.stderr"),
        }
    ]
}
