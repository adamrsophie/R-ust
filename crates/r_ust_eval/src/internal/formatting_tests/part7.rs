use super::{check, Row};

#[test]
fn sprintf_unused_formats() {
    check(Row {
        id: "sprintf_unused_formats",
        source: r#"sprintf(c('one','two'),1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_unused_formats.stdout"),
        stderr: include_str!("raw/sprintf_unused_formats.stderr"),
    });
}

#[test]
fn sprintf_type_symbol() {
    check(Row {
        id: "sprintf_type_symbol",
        source: r#"sprintf('%s',quote(x))"#,
        error: true,
        stdout: include_str!("raw/sprintf_type_symbol.stdout"),
        stderr: include_str!("raw/sprintf_type_symbol.stderr"),
    });
}

#[test]
fn sprintf_type_call() {
    check(Row {
        id: "sprintf_type_call",
        source: r#"sprintf('%s',quote(x+1))"#,
        error: true,
        stdout: include_str!("raw/sprintf_type_call.stdout"),
        stderr: include_str!("raw/sprintf_type_call.stderr"),
    });
}

#[test]
fn sprintf_long_result() {
    check(Row {
        id: "sprintf_long_result",
        source: r#"sprintf('%8193s','x')"#,
        error: true,
        stdout: include_str!("raw/sprintf_long_result.stdout"),
        stderr: include_str!("raw/sprintf_long_result.stderr"),
    });
}

#[test]
fn sprintf_coercion_order_integer_float() {
    check(Row {
        id: "sprintf_coercion_order_integer_float",
        source: r#"sprintf('%1$d %1$f',2L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_coercion_order_integer_float.stdout"),
        stderr: include_str!("raw/sprintf_coercion_order_integer_float.stderr"),
    });
}

#[test]
fn sprintf_coercion_order_float_integer() {
    check(Row {
        id: "sprintf_coercion_order_float_integer",
        source: r#"sprintf('%1$f %1$d',2L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_coercion_order_float_integer.stdout"),
        stderr: include_str!("raw/sprintf_coercion_order_float_integer.stderr"),
    });
}

#[test]
fn sprintf_format_row_coercion() {
    check(Row {
        id: "sprintf_format_row_coercion",
        source: r#"sprintf(c('%f','%d'),c(1L,2L))"#,
        error: true,
        stdout: include_str!("raw/sprintf_format_row_coercion.stdout"),
        stderr: include_str!("raw/sprintf_format_row_coercion.stderr"),
    });
}

#[test]
fn sprintf_list_float() {
    check(Row {
        id: "sprintf_list_float",
        source: r#"sprintf('%.1f',list(1L,2))"#,
        error: false,
        stdout: include_str!("raw/sprintf_list_float.stdout"),
        stderr: include_str!("raw/sprintf_list_float.stderr"),
    });
}
