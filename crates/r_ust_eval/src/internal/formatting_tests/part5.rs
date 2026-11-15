use super::{check, Row};

#[test]
fn sprintf_string_coerce() {
    check(Row {
        id: "sprintf_string_coerce",
        source: r#"sprintf('%s',list(1L,quote(x+1L),c('a','b')))"#,
        error: false,
        stdout: include_str!("raw/sprintf_string_coerce.stdout"),
        stderr: include_str!("raw/sprintf_string_coerce.stderr"),
    });
}

#[test]
fn sprintf_string_utf8() {
    check(Row {
        id: "sprintf_string_utf8",
        source: r#"sprintf(c('%s','%6s','%.2s'),'é')"#,
        error: false,
        stdout: include_str!("raw/sprintf_string_utf8.stdout"),
        stderr: include_str!("raw/sprintf_string_utf8.stderr"),
    });
}

#[test]
fn sprintf_position() {
    check(Row {
        id: "sprintf_position",
        source: r#"sprintf('%2$s %1$d %2$s',7L,'word')"#,
        error: false,
        stdout: include_str!("raw/sprintf_position.stdout"),
        stderr: include_str!("raw/sprintf_position.stderr"),
    });
}

#[test]
fn sprintf_position_mixed() {
    check(Row {
        id: "sprintf_position_mixed",
        source: r#"sprintf('%2$s %s','first','second')"#,
        error: false,
        stdout: include_str!("raw/sprintf_position_mixed.stdout"),
        stderr: include_str!("raw/sprintf_position_mixed.stderr"),
    });
}

#[test]
fn sprintf_position_missing() {
    check(Row {
        id: "sprintf_position_missing",
        source: r#"sprintf('%2$s','first')"#,
        error: true,
        stdout: include_str!("raw/sprintf_position_missing.stdout"),
        stderr: include_str!("raw/sprintf_position_missing.stderr"),
    });
}

#[test]
fn sprintf_position_zero() {
    check(Row {
        id: "sprintf_position_zero",
        source: r#"sprintf('%0$s','first')"#,
        error: true,
        stdout: include_str!("raw/sprintf_position_zero.stdout"),
        stderr: include_str!("raw/sprintf_position_zero.stderr"),
    });
}

#[test]
fn sprintf_width_star() {
    check(Row {
        id: "sprintf_width_star",
        source: r#"sprintf('%*s',c(6L,-6L),'abc')"#,
        error: false,
        stdout: include_str!("raw/sprintf_width_star.stdout"),
        stderr: include_str!("raw/sprintf_width_star.stderr"),
    });
}

#[test]
fn sprintf_precision_star() {
    check(Row {
        id: "sprintf_precision_star",
        source: r#"sprintf('%.*f',c(0L,2L),1.25)"#,
        error: false,
        stdout: include_str!("raw/sprintf_precision_star.stdout"),
        stderr: include_str!("raw/sprintf_precision_star.stderr"),
    });
}
