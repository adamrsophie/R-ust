use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "get_missing_argument_class",
            setup: "",
            source: "f<-function(x)tryCatch(get('x'),error=function(e)class(e));f()",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/get_missing_argument_class.stdout"),
            stderr: include_str!("../raw/get_missing_argument_class.stderr"),
        },
        Row {
            id: "as_name_roundtrip",
            setup: "",
            source: "list(is.name(as.name('x')),is.symbol(as.symbol(7L)),as.character(as.name('x')),identical(as.name,as.symbol))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_name_roundtrip.stdout"),
            stderr: include_str!("../raw/as_name_roundtrip.stderr"),
        },
        Row {
            id: "as_name_empty",
            setup: "",
            source: "tryCatch(as.name(''),error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_name_empty.stdout"),
            stderr: include_str!("../raw/as_name_empty.stderr"),
        },
        Row {
            id: "as_call_list_tags",
            setup: "",
            source: "as.call(list(quote(f),x=quote(a),quote(b)))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_call_list_tags.stdout"),
            stderr: include_str!("../raw/as_call_list_tags.stderr"),
        }
    ]
}
