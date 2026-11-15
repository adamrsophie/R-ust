use super::{check, Row};

#[test]
fn deparse_names_without_attr() {
    check(Row {
        id: "deparse_names_without_attr",
        source: r#"x<-list(1L,2L);names(x)<-c(NA_character_,'');deparse(x,control='niceNames')"#,
        error: false,
        stdout: include_str!("raw/deparse_names_without_attr.stdout"),
        stderr: include_str!("raw/deparse_names_without_attr.stderr"),
    });
}

#[test]
fn deparse_control_errors_plural() {
    check(Row {
        id: "deparse_control_errors_plural",
        source: r#"deparse(1L,control=c('bad','worse'))"#,
        error: true,
        stdout: include_str!("raw/deparse_control_errors_plural.stdout"),
        stderr: include_str!("raw/deparse_control_errors_plural.stderr"),
    });
}

#[test]
fn sprintf_large_precision_fixed() {
    check(Row {
        id: "sprintf_large_precision_fixed",
        source: r#"sprintf('%.8193f',1)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_precision_fixed.stdout"),
        stderr: include_str!("raw/sprintf_large_precision_fixed.stderr"),
    });
}

#[test]
fn sprintf_large_precision_exponent() {
    check(Row {
        id: "sprintf_large_precision_exponent",
        source: r#"sprintf('%.8193e',1)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_precision_exponent.stdout"),
        stderr: include_str!("raw/sprintf_large_precision_exponent.stderr"),
    });
}

#[test]
fn sprintf_large_precision_hex() {
    check(Row {
        id: "sprintf_large_precision_hex",
        source: r#"sprintf('%.8193a',1)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_precision_hex.stdout"),
        stderr: include_str!("raw/sprintf_large_precision_hex.stderr"),
    });
}

#[test]
fn sprintf_long_string_precision() {
    check(Row {
        id: "sprintf_long_string_precision",
        source: r#"sprintf('%.3s',paste(rep('x',8193L),collapse=''))"#,
        error: false,
        stdout: include_str!("raw/sprintf_long_string_precision.stdout"),
        stderr: include_str!("raw/sprintf_long_string_precision.stderr"),
    });
}

#[test]
fn sprintf_large_integer_sign() {
    check(Row {
        id: "sprintf_large_integer_sign",
        source: r#"sprintf('%.8193d',-1L)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_integer_sign.stdout"),
        stderr: include_str!("raw/sprintf_large_integer_sign.stderr"),
    });
}

#[test]
fn sprintf_large_integer_hex_prefix() {
    check(Row {
        id: "sprintf_large_integer_hex_prefix",
        source: r#"sprintf('%#.8193x',1L)"#,
        error: true,
        stdout: include_str!("raw/sprintf_large_integer_hex_prefix.stdout"),
        stderr: include_str!("raw/sprintf_large_integer_hex_prefix.stderr"),
    });
}
