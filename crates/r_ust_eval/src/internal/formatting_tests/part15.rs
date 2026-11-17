use super::{check, Row};

#[test]
fn squote_empty() {
    check(Row {
        id: "squote_empty",
        source: r#"sQuote(character())"#,
        error: false,
        stdout: include_str!("raw/squote_empty.stdout"),
        stderr: include_str!("raw/squote_empty.stderr"),
    });
}

#[test]
fn squote_true() {
    check(Row {
        id: "squote_true",
        source: r#"sQuote('x',TRUE)"#,
        error: false,
        stdout: include_str!("raw/squote_true.stdout"),
        stderr: include_str!("raw/squote_true.stderr"),
    });
}

#[test]
fn squote_false() {
    check(Row {
        id: "squote_false",
        source: r#"sQuote('x',FALSE)"#,
        error: false,
        stdout: include_str!("raw/squote_false.stdout"),
        stderr: include_str!("raw/squote_false.stderr"),
    });
}

#[test]
fn squote_tex() {
    check(Row {
        id: "squote_tex",
        source: r#"sQuote('x','TeX')"#,
        error: false,
        stdout: include_str!("raw/squote_tex.stdout"),
        stderr: include_str!("raw/squote_tex.stderr"),
    });
}

#[test]
fn squote_utf8() {
    check(Row {
        id: "squote_utf8",
        source: r#"sQuote('x','UTF-8')"#,
        error: false,
        stdout: include_str!("raw/squote_utf8.stdout"),
        stderr: include_str!("raw/squote_utf8.stderr"),
    });
}

#[test]
fn squote_custom() {
    check(Row {
        id: "squote_custom",
        source: r#"sQuote(c('a','b'),c('[',']','{','}'))"#,
        error: false,
        stdout: include_str!("raw/squote_custom.stdout"),
        stderr: include_str!("raw/squote_custom.stderr"),
    });
}

#[test]
fn locale_c() {
    check(Row {
        id: "locale_c",
        source: r#"c(names(l10n_info()),unlist(lapply(l10n_info(),as.character)))"#,
        error: false,
        stdout: if cfg!(target_os = "linux") {
            include_str!("raw/locale_c_linux.stdout")
        } else {
            include_str!("raw/locale_c.stdout")
        },
        stderr: include_str!("raw/locale_c.stderr"),
    });
}

#[test]
fn deparse_matrix() {
    check(Row {
        id: "deparse_matrix",
        source: r#"deparse(matrix(1:4,2L))"#,
        error: false,
        stdout: include_str!("raw/deparse_matrix.stdout"),
        stderr: include_str!("raw/deparse_matrix.stderr"),
    });
}
