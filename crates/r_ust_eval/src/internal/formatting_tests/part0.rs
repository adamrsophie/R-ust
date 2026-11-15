use super::{check, Row};

#[test]
fn sprintf_text() {
    check(Row {
        id: "sprintf_text",
        source: r#"sprintf('value=%s!',c('one','two'))"#,
        error: false,
        stdout: include_str!("raw/sprintf_text.stdout"),
        stderr: include_str!("raw/sprintf_text.stderr"),
    });
}

#[test]
fn sprintf_percent() {
    check(Row {
        id: "sprintf_percent",
        source: r#"sprintf('%% %s %%','x')"#,
        error: false,
        stdout: include_str!("raw/sprintf_percent.stdout"),
        stderr: include_str!("raw/sprintf_percent.stderr"),
    });
}

#[test]
fn sprintf_format_na() {
    check(Row {
        id: "sprintf_format_na",
        source: r#"sprintf(c(NA_character_,'%s'),c('a','b'))"#,
        error: false,
        stdout: include_str!("raw/sprintf_format_na.stdout"),
        stderr: include_str!("raw/sprintf_format_na.stderr"),
    });
}

#[test]
fn sprintf_format_empty() {
    check(Row {
        id: "sprintf_format_empty",
        source: r#"sprintf(character(),1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_format_empty.stdout"),
        stderr: include_str!("raw/sprintf_format_empty.stderr"),
    });
}

#[test]
fn sprintf_argument_empty() {
    check(Row {
        id: "sprintf_argument_empty",
        source: r#"sprintf('%d',integer())"#,
        error: false,
        stdout: include_str!("raw/sprintf_argument_empty.stdout"),
        stderr: include_str!("raw/sprintf_argument_empty.stderr"),
    });
}

#[test]
fn sprintf_argument_null() {
    check(Row {
        id: "sprintf_argument_null",
        source: r#"sprintf('%s',NULL)"#,
        error: false,
        stdout: include_str!("raw/sprintf_argument_null.stdout"),
        stderr: include_str!("raw/sprintf_argument_null.stderr"),
    });
}

#[test]
fn sprintf_recycle() {
    check(Row {
        id: "sprintf_recycle",
        source: r#"sprintf(c('%d','%04d'),1:4)"#,
        error: false,
        stdout: include_str!("raw/sprintf_recycle.stdout"),
        stderr: include_str!("raw/sprintf_recycle.stderr"),
    });
}

#[test]
fn sprintf_recycle_invalid() {
    check(Row {
        id: "sprintf_recycle_invalid",
        source: r#"sprintf(c('%d','%d'),1:3)"#,
        error: true,
        stdout: include_str!("raw/sprintf_recycle_invalid.stdout"),
        stderr: include_str!("raw/sprintf_recycle_invalid.stderr"),
    });
}
