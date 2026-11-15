use super::{check, Row};

#[test]
fn sprintf_precision_negative() {
    check(Row {
        id: "sprintf_precision_negative",
        source: r#"sprintf('%.*f',-2L,1.25)"#,
        error: false,
        stdout: include_str!("raw/sprintf_precision_negative.stdout"),
        stderr: include_str!("raw/sprintf_precision_negative.stderr"),
    });
}

#[test]
fn sprintf_star_position() {
    check(Row {
        id: "sprintf_star_position",
        source: r#"sprintf('%2$*1$s',6L,'abc')"#,
        error: false,
        stdout: include_str!("raw/sprintf_star_position.stdout"),
        stderr: include_str!("raw/sprintf_star_position.stderr"),
    });
}

#[test]
fn sprintf_star_fraction() {
    check(Row {
        id: "sprintf_star_fraction",
        source: r#"sprintf('%*s',3.9,'a')"#,
        error: false,
        stdout: include_str!("raw/sprintf_star_fraction.stdout"),
        stderr: include_str!("raw/sprintf_star_fraction.stderr"),
    });
}

#[test]
fn sprintf_star_logical_error() {
    check(Row {
        id: "sprintf_star_logical_error",
        source: r#"sprintf('%*s',TRUE,'a')"#,
        error: true,
        stdout: include_str!("raw/sprintf_star_logical_error.stdout"),
        stderr: include_str!("raw/sprintf_star_logical_error.stderr"),
    });
}

#[test]
fn sprintf_star_na_error() {
    check(Row {
        id: "sprintf_star_na_error",
        source: r#"sprintf('%*s',NA_integer_,'a')"#,
        error: true,
        stdout: include_str!("raw/sprintf_star_na_error.stdout"),
        stderr: include_str!("raw/sprintf_star_na_error.stderr"),
    });
}

#[test]
fn sprintf_star_two_error() {
    check(Row {
        id: "sprintf_star_two_error",
        source: r#"sprintf('%*.*f',4L,2L,1)"#,
        error: true,
        stdout: include_str!("raw/sprintf_star_two_error.stdout"),
        stderr: include_str!("raw/sprintf_star_two_error.stderr"),
    });
}

#[test]
fn sprintf_unused_one() {
    check(Row {
        id: "sprintf_unused_one",
        source: r#"sprintf('%s','a','b')"#,
        error: false,
        stdout: include_str!("raw/sprintf_unused_one.stdout"),
        stderr: include_str!("raw/sprintf_unused_one.stderr"),
    });
}

#[test]
fn sprintf_unused_many() {
    check(Row {
        id: "sprintf_unused_many",
        source: r#"sprintf('fixed',1L,2L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_unused_many.stdout"),
        stderr: include_str!("raw/sprintf_unused_many.stderr"),
    });
}
