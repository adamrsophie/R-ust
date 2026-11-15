use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "do_call_order",
            setup: "",
            source: "do.call(function(x,y){y;x},list(quote({cat('X|');1L}),quote({cat('Y|');2L})))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_order.stdout"),
            stderr: include_str!("../raw/do_call_order.stderr"),
        },
        Row {
            id: "do_call_named_match",
            setup: "",
            source: "do.call(function(x,y=2L)x+y,list(y=3L,x=7L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_named_match.stdout"),
            stderr: include_str!("../raw/do_call_named_match.stderr"),
        },
        Row {
            id: "do_call_primitive",
            setup: "",
            source: "do.call('+',list(2L,3L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_primitive.stdout"),
            stderr: include_str!("../raw/do_call_primitive.stderr"),
        },
        Row {
            id: "do_call_visibility",
            setup: "",
            source: "do.call(function(x)invisible(x),list(7L))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/do_call_visibility.stdout"),
            stderr: include_str!("../raw/do_call_visibility.stderr"),
        },
    ]
}
