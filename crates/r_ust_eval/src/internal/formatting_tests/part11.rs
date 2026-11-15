use super::{check, Row};

#[test]
fn deparse_control_keepna() {
    check(Row {
        id: "deparse_control_keepna",
        source: r#"deparse(c(1L,NA_integer_),control='keepNA')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_keepna.stdout"),
        stderr: include_str!("raw/deparse_control_keepna.stderr"),
    });
}

#[test]
fn deparse_control_keepinteger() {
    check(Row {
        id: "deparse_control_keepinteger",
        source: r#"deparse(c(1L,NA_integer_),control='keepInteger')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_keepinteger.stdout"),
        stderr: include_str!("raw/deparse_control_keepinteger.stderr"),
    });
}

#[test]
fn deparse_control_quote() {
    check(Row {
        id: "deparse_control_quote",
        source: r#"deparse(quote(x+1L),control='quoteExpressions')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_quote.stdout"),
        stderr: include_str!("raw/deparse_control_quote.stderr"),
    });
}

#[test]
fn deparse_control_hex() {
    check(Row {
        id: "deparse_control_hex",
        source: r#"deparse(c(0.1,1,5e-324),control='hexNumeric')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_hex.stdout"),
        stderr: include_str!("raw/deparse_control_hex.stderr"),
    });
}

#[test]
fn deparse_control_digits17() {
    check(Row {
        id: "deparse_control_digits17",
        source: r#"deparse(c(0.1,1.2345678901234567),control='digits17')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_digits17.stdout"),
        stderr: include_str!("raw/deparse_control_digits17.stderr"),
    });
}

#[test]
fn deparse_control_all() {
    check(Row {
        id: "deparse_control_all",
        source: r#"deparse(list(x=1L),control='all')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_all.stdout"),
        stderr: include_str!("raw/deparse_control_all.stderr"),
    });
}

#[test]
fn deparse_control_exact() {
    check(Row {
        id: "deparse_control_exact",
        source: r#"deparse(list(x=0.1),control='exact')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_exact.stdout"),
        stderr: include_str!("raw/deparse_control_exact.stderr"),
    });
}

#[test]
fn deparse_control_prefix() {
    check(Row {
        id: "deparse_control_prefix",
        source: r#"deparse(1L,control='keepI')"#,
        error: false,
        stdout: include_str!("raw/deparse_control_prefix.stdout"),
        stderr: include_str!("raw/deparse_control_prefix.stderr"),
    });
}
