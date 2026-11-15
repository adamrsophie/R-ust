use super::{check, Row};

#[test]
fn sprintf_empty_unused() {
    check(Row {
        id: "sprintf_empty_unused",
        source: r#"sprintf('fixed',character())"#,
        error: false,
        stdout: include_str!("raw/sprintf_empty_unused.stdout"),
        stderr: include_str!("raw/sprintf_empty_unused.stderr"),
    });
}

#[test]
fn deparse_null() {
    check(Row {
        id: "deparse_null",
        source: r#"deparse(NULL)"#,
        error: false,
        stdout: include_str!("raw/deparse_null.stdout"),
        stderr: include_str!("raw/deparse_null.stderr"),
    });
}

#[test]
fn deparse_empty() {
    check(Row {
        id: "deparse_empty",
        source: r#"deparse(character())"#,
        error: false,
        stdout: include_str!("raw/deparse_empty.stdout"),
        stderr: include_str!("raw/deparse_empty.stderr"),
    });
}

#[test]
fn deparse_character() {
    check(Row {
        id: "deparse_character",
        source: r#"deparse(c('one',NA_character_,'line\nbreak','a"b','a\\b'))"#,
        error: false,
        stdout: include_str!("raw/deparse_character.stdout"),
        stderr: include_str!("raw/deparse_character.stderr"),
    });
}

#[test]
fn deparse_integer() {
    check(Row {
        id: "deparse_integer",
        source: r#"deparse(c(1L,3L,NA_integer_))"#,
        error: false,
        stdout: include_str!("raw/deparse_integer.stdout"),
        stderr: include_str!("raw/deparse_integer.stderr"),
    });
}

#[test]
fn deparse_integer_sequence() {
    check(Row {
        id: "deparse_integer_sequence",
        source: r#"deparse(1:5)"#,
        error: false,
        stdout: include_str!("raw/deparse_integer_sequence.stdout"),
        stderr: include_str!("raw/deparse_integer_sequence.stderr"),
    });
}

#[test]
fn deparse_real() {
    check(Row {
        id: "deparse_real",
        source: r#"deparse(c(1,0.1,NA_real_,NaN,Inf,-Inf))"#,
        error: false,
        stdout: include_str!("raw/deparse_real.stdout"),
        stderr: include_str!("raw/deparse_real.stderr"),
    });
}

#[test]
fn deparse_logical() {
    check(Row {
        id: "deparse_logical",
        source: r#"deparse(c(TRUE,FALSE,NA))"#,
        error: false,
        stdout: include_str!("raw/deparse_logical.stdout"),
        stderr: include_str!("raw/deparse_logical.stderr"),
    });
}
