use super::{check, Row};

#[test]
fn sprintf_large_integer_octal_prefix() {
    check(Row {
        id: "sprintf_large_integer_octal_prefix",
        source: r#"sprintf('%#.8193o',1L)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_integer_octal_prefix.stdout"),
        stderr: include_str!("raw/sprintf_large_integer_octal_prefix.stderr"),
    });
}

#[test]
fn sprintf_rebound_character() {
    check(Row {
        id: "sprintf_rebound_character",
        source: r#"as.character<-function(x)'changed';sprintf('%s',1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_rebound_character.stdout"),
        stderr: include_str!("raw/sprintf_rebound_character.stderr"),
    });
}

#[test]
fn sprintf_rebound_double() {
    check(Row {
        id: "sprintf_rebound_double",
        source: r#"as.double<-function(x)2.5;sprintf('%.1f',1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_rebound_double.stdout"),
        stderr: include_str!("raw/sprintf_rebound_double.stderr"),
    });
}

#[test]
fn sprintf_rebound_character_empty() {
    check(Row {
        id: "sprintf_rebound_character_empty",
        source: r#"as.character<-function(x)character();sprintf('%s',1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_rebound_character_empty.stdout"),
        stderr: include_str!("raw/sprintf_rebound_character_empty.stderr"),
    });
}

#[test]
fn sprintf_rebound_character_order() {
    check(Row {
        id: "sprintf_rebound_character_order",
        source: r#"n<-0L;as.character<-function(x){n<<-n+1L;'changed'};c(sprintf('%1$s %1$s',1L),as.character(n))"#,
        error: false,
        stdout: include_str!("raw/sprintf_rebound_character_order.stdout"),
        stderr: include_str!("raw/sprintf_rebound_character_order.stderr"),
    });
}

#[test]
fn deparse_nlines_skips_body() {
    check(Row {
        id: "deparse_nlines_skips_body",
        source: r#"deparse(function(x)stop('unforced'),nlines=1L)"#,
        error: false,
        stdout: include_str!("raw/deparse_nlines_skips_body.stdout"),
        stderr: include_str!("raw/deparse_nlines_skips_body.stderr"),
    });
}

#[test]
fn deparse_warning_width() {
    check(Row {
        id: "deparse_warning_width",
        source: r#"warning('text',call.=FALSE);'done'"#,
        error: false,
        stdout: include_str!("raw/deparse_warning_width.stdout"),
        stderr: include_str!("raw/deparse_warning_width.stderr"),
    });
}
