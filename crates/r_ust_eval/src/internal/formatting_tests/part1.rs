use super::{check, Row};

#[test]
fn sprintf_no_args() {
    check(Row {
        id: "sprintf_no_args",
        source: r#"sprintf('literal %%')"#,
        error: false,
        stdout: include_str!("raw/sprintf_no_args.stdout"),
        stderr: include_str!("raw/sprintf_no_args.stderr"),
    });
}

#[test]
fn sprintf_too_few() {
    check(Row {
        id: "sprintf_too_few",
        source: r#"sprintf('%s %s','one')"#,
        error: true,
        stdout: include_str!("raw/sprintf_too_few.stdout"),
        stderr: include_str!("raw/sprintf_too_few.stderr"),
    });
}

#[test]
fn sprintf_fmt_type() {
    check(Row {
        id: "sprintf_fmt_type",
        source: r#"sprintf(1L,1L)"#,
        error: true,
        stdout: include_str!("raw/sprintf_fmt_type.stdout"),
        stderr: include_str!("raw/sprintf_fmt_type.stderr"),
    });
}

#[test]
fn sprintf_bad_spec() {
    check(Row {
        id: "sprintf_bad_spec",
        source: r#"sprintf('%q',1L)"#,
        error: true,
        stdout: include_str!("raw/sprintf_bad_spec.stdout"),
        stderr: include_str!("raw/sprintf_bad_spec.stderr"),
    });
}

#[test]
fn sprintf_trailing_percent() {
    check(Row {
        id: "sprintf_trailing_percent",
        source: r#"sprintf('text%')"#,
        error: true,
        stdout: include_str!("raw/sprintf_trailing_percent.stdout"),
        stderr: include_str!("raw/sprintf_trailing_percent.stderr"),
    });
}

#[test]
fn sprintf_integer() {
    check(Row {
        id: "sprintf_integer",
        source: r#"sprintf('%d',c(0L,1L,-1L,2147483647L,NA_integer_))"#,
        error: false,
        stdout: include_str!("raw/sprintf_integer.stdout"),
        stderr: include_str!("raw/sprintf_integer.stderr"),
    });
}

#[test]
fn sprintf_logical() {
    check(Row {
        id: "sprintf_logical",
        source: r#"sprintf('%i',c(TRUE,FALSE,NA))"#,
        error: false,
        stdout: include_str!("raw/sprintf_logical.stdout"),
        stderr: include_str!("raw/sprintf_logical.stderr"),
    });
}

#[test]
fn sprintf_logical_hex_error() {
    check(Row {
        id: "sprintf_logical_hex_error",
        source: r#"sprintf('%x',TRUE)"#,
        error: true,
        stdout: include_str!("raw/sprintf_logical_hex_error.stdout"),
        stderr: include_str!("raw/sprintf_logical_hex_error.stderr"),
    });
}
