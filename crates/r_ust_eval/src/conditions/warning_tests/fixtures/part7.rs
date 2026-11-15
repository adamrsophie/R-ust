use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "restart_list_spec",
            setup: "",
            source: "withRestarts(invokeRestart('r',5L),r=list(handler=function(x)x+2L,description='restart'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_list_spec.stdout"),
            stderr: include_str!("../raw/restart_list_spec.stderr"),
        },
        Row {
            id: "restart_character_handler",
            setup: "",
            source: "withRestarts(invokeRestart('r',5L),r=list(handler='identity'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_character_handler.stdout"),
            stderr: include_str!("../raw/restart_character_handler.stderr"),
        },
        Row {
            id: "restart_symbol_handler",
            setup: "",
            source: "withRestarts(invokeRestart('r',5L),r=list(handler=as.name('identity')))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_symbol_handler.stdout"),
            stderr: include_str!("../raw/restart_symbol_handler.stderr"),
        },
        Row {
            id: "restart_description_spec",
            setup: "",
            source: "withRestarts(findRestart('r')$description,r='description')",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/restart_description_spec.stdout"),
            stderr: include_str!("../raw/restart_description_spec.stderr"),
        }
    ]
}
