use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "as_call_character_head",
            setup: "",
            source: "as.call(list('f',quote(x)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_call_character_head.stdout"),
            stderr: include_str!("../raw/as_call_character_head.stderr"),
        },
        Row {
            id: "as_call_pairlist",
            setup: "",
            source: "as.call(pairlist(quote(f),x=1L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_call_pairlist.stdout"),
            stderr: include_str!("../raw/as_call_pairlist.stderr"),
        },
        Row {
            id: "as_call_expression",
            setup: "",
            source: "as.call(expression(f,x=1L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_call_expression.stdout"),
            stderr: include_str!("../raw/as_call_expression.stderr"),
        },
        Row {
            id: "as_call_empty",
            setup: "",
            source: "tryCatch(as.call(list()),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_call_empty.stdout"),
            stderr: include_str!("../raw/as_call_empty.stderr"),
        },
    ]
}
