use super::Row;
pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            id: "enquote_shadowed_quote",
            setup: "",
            source: "quote<-function(...)stop('SHADOW');enquote(as.name('x'))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/enquote_shadowed_quote.stdout"),
            stderr: include_str!("../raw/enquote_shadowed_quote.stderr"),
        },
        Row {
            id: "namespace_real_bindings",
            setup: "",
            source: "list(identical(base::quote,.Primitive('quote')),identical(base:::quote,base::quote),identical('base'::'quote',base::quote))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/namespace_real_bindings.stdout"),
            stderr: include_str!("../raw/namespace_real_bindings.stderr"),
        },
        Row {
            id: "namespace_missing_binding",
            setup: "",
            source: "tryCatch(base::not_a_real_binding,error=function(e)conditionMessage(e))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/namespace_missing_binding.stdout"),
            stderr: include_str!("../raw/namespace_missing_binding.stderr"),
        },
        Row {
            id: "as_environment_identity",
            setup: "",
            source: "list(identical(as.environment('.GlobalEnv'),globalenv()),identical(as.environment('package:base'),baseenv()),identical(as.environment(1L),globalenv()),identical(as.environment(2L),.AutoloadEnv),identical(as.environment(3L),baseenv()),identical(as.environment(4L),emptyenv()))",
            binding: "",
            replacement: "",
            error_message: "",
            status: 0,
            stdout: include_str!("../raw/as_environment_identity.stdout"),
            stderr: include_str!("../raw/as_environment_identity.stderr"),
        }
    ]
}
