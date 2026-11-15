use super::{check, Row};

#[test]
fn deparse_control_bad() {
    check(Row {
        id: "deparse_control_bad",
        source: r#"deparse(1L,control='bad')"#,
        error: true,
        stdout: include_str!("raw/deparse_control_bad.stdout"),
        stderr: include_str!("raw/deparse_control_bad.stderr"),
    });
}

#[test]
fn deparse_control_conflict() {
    check(Row {
        id: "deparse_control_conflict",
        source: r#"deparse(1L,control=c('hexNumeric','digits17'))"#,
        error: true,
        stdout: include_str!("raw/deparse_control_conflict.stdout"),
        stderr: include_str!("raw/deparse_control_conflict.stderr"),
    });
}

#[test]
fn deparse_control_all_exact() {
    check(Row {
        id: "deparse_control_all_exact",
        source: r#"deparse(1L,control=c('all','exact'))"#,
        error: true,
        stdout: include_str!("raw/deparse_control_all_exact.stdout"),
        stderr: include_str!("raw/deparse_control_all_exact.stderr"),
    });
}

#[test]
fn deparse_width() {
    check(Row {
        id: "deparse_width",
        source: r#"deparse(quote(f(aaaaaaaaaa,bbbbbbbbbb,cccccccccc,dddddddddd)),width.cutoff=20L)"#,
        error: false,
        stdout: include_str!("raw/deparse_width.stdout"),
        stderr: include_str!("raw/deparse_width.stderr"),
    });
}

#[test]
fn deparse_width_bad() {
    check(Row {
        id: "deparse_width_bad",
        source: r#"deparse(quote(f(x)),width.cutoff=19L)"#,
        error: false,
        stdout: include_str!("raw/deparse_width_bad.stdout"),
        stderr: include_str!("raw/deparse_width_bad.stderr"),
    });
}

#[test]
fn deparse_width_na() {
    check(Row {
        id: "deparse_width_na",
        source: r#"deparse(quote(f(x)),width.cutoff=NA_integer_)"#,
        error: false,
        stdout: include_str!("raw/deparse_width_na.stdout"),
        stderr: include_str!("raw/deparse_width_na.stderr"),
    });
}

#[test]
fn deparse_nlines() {
    check(Row {
        id: "deparse_nlines",
        source: r#"deparse(quote({a;b;c}),nlines=2L)"#,
        error: false,
        stdout: include_str!("raw/deparse_nlines.stdout"),
        stderr: include_str!("raw/deparse_nlines.stderr"),
    });
}

#[test]
fn deparse_nlines_zero() {
    check(Row {
        id: "deparse_nlines_zero",
        source: r#"deparse(quote({a;b;c}),nlines=0L)"#,
        error: false,
        stdout: include_str!("raw/deparse_nlines_zero.stdout"),
        stderr: include_str!("raw/deparse_nlines_zero.stderr"),
    });
}
