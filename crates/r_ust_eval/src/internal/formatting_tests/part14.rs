use super::{check, Row};

#[test]
fn ngettext_many() {
    check(Row {
        id: "ngettext_many",
        source: r#"ngettext(2L,'one','many')"#,
        error: false,
        stdout: include_str!("raw/ngettext_many.stdout"),
        stderr: include_str!("raw/ngettext_many.stderr"),
    });
}

#[test]
fn ngettext_fraction() {
    check(Row {
        id: "ngettext_fraction",
        source: r#"ngettext(1.9,'one','many')"#,
        error: false,
        stdout: include_str!("raw/ngettext_fraction.stdout"),
        stderr: include_str!("raw/ngettext_fraction.stderr"),
    });
}

#[test]
fn ngettext_na() {
    check(Row {
        id: "ngettext_na",
        source: r#"ngettext(NA_integer_,'one','many')"#,
        error: true,
        stdout: include_str!("raw/ngettext_na.stdout"),
        stderr: include_str!("raw/ngettext_na.stderr"),
    });
}

#[test]
fn ngettext_negative() {
    check(Row {
        id: "ngettext_negative",
        source: r#"ngettext(-1L,'one','many')"#,
        error: true,
        stdout: include_str!("raw/ngettext_negative.stdout"),
        stderr: include_str!("raw/ngettext_negative.stderr"),
    });
}

#[test]
fn ngettext_bad_msg1() {
    check(Row {
        id: "ngettext_bad_msg1",
        source: r#"ngettext(1L,1L,'many')"#,
        error: true,
        stdout: include_str!("raw/ngettext_bad_msg1.stdout"),
        stderr: include_str!("raw/ngettext_bad_msg1.stderr"),
    });
}

#[test]
fn ngettext_bad_msg2() {
    check(Row {
        id: "ngettext_bad_msg2",
        source: r#"ngettext(2L,'one',c('many','more'))"#,
        error: true,
        stdout: include_str!("raw/ngettext_bad_msg2.stdout"),
        stderr: include_str!("raw/ngettext_bad_msg2.stderr"),
    });
}

#[test]
fn ngettext_na_message() {
    check(Row {
        id: "ngettext_na_message",
        source: r#"ngettext(1L,NA_character_,'many')"#,
        error: false,
        stdout: include_str!("raw/ngettext_na_message.stdout"),
        stderr: include_str!("raw/ngettext_na_message.stderr"),
    });
}

#[test]
fn squote_default() {
    check(Row {
        id: "squote_default",
        source: r#"sQuote(c('x','a b',NA_character_))"#,
        error: false,
        stdout: include_str!("raw/squote_default.stdout"),
        stderr: include_str!("raw/squote_default.stderr"),
    });
}
