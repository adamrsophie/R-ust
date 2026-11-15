use super::{check, Row};

#[test]
fn deparse_nlines_na() {
    check(Row {
        id: "deparse_nlines_na",
        source: r#"deparse(quote({a;b;c}),nlines=NA_integer_)"#,
        error: false,
        stdout: include_str!("raw/deparse_nlines_na.stdout"),
        stderr: include_str!("raw/deparse_nlines_na.stderr"),
    });
}

#[test]
fn deparse_backtick_na() {
    check(Row {
        id: "deparse_backtick_na",
        source: r#"deparse(quote(x),backtick=NA)"#,
        error: true,
        stdout: include_str!("raw/deparse_backtick_na.stdout"),
        stderr: include_str!("raw/deparse_backtick_na.stderr"),
    });
}

#[test]
fn deparse_unicode() {
    check(Row {
        id: "deparse_unicode",
        source: r#"deparse(c('é','中'))"#,
        error: false,
        stdout: include_str!("raw/deparse_unicode.stdout"),
        stderr: include_str!("raw/deparse_unicode.stderr"),
    });
}

#[test]
fn consumer_restart_name() {
    check(Row {
        id: "consumer_restart_name",
        source: r#"tryCatch(invokeRestart('unavailable'),error=function(e)conditionMessage(e))"#,
        error: false,
        stdout: include_str!("raw/consumer_restart_name.stdout"),
        stderr: include_str!("raw/consumer_restart_name.stderr"),
    });
}

#[test]
fn consumer_match_fun_invalid() {
    check(Row {
        id: "consumer_match_fun_invalid",
        source: r#"tryCatch(match.fun(1L),error=function(e)conditionMessage(e))"#,
        error: false,
        stdout: include_str!("raw/consumer_match_fun_invalid.stdout"),
        stderr: include_str!("raw/consumer_match_fun_invalid.stderr"),
    });
}

#[test]
fn consumer_match_fun_descend() {
    check(Row {
        id: "consumer_match_fun_descend",
        source: r#"identity<-7L;tryCatch(match.fun('identity',descend=FALSE),error=function(e)conditionMessage(e))"#,
        error: false,
        stdout: include_str!("raw/consumer_match_fun_descend.stdout"),
        stderr: include_str!("raw/consumer_match_fun_descend.stderr"),
    });
}

#[test]
fn ngettext_one() {
    check(Row {
        id: "ngettext_one",
        source: r#"ngettext(1L,'one','many')"#,
        error: false,
        stdout: include_str!("raw/ngettext_one.stdout"),
        stderr: include_str!("raw/ngettext_one.stderr"),
    });
}

#[test]
fn ngettext_zero() {
    check(Row {
        id: "ngettext_zero",
        source: r#"ngettext(0L,'one','many')"#,
        error: false,
        stdout: include_str!("raw/ngettext_zero.stdout"),
        stderr: include_str!("raw/ngettext_zero.stderr"),
    });
}
