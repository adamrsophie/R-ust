use super::{check, Row};

#[test]
fn sprintf_logical_float() {
    check(Row {
        id: "sprintf_logical_float",
        source: r#"sprintf('%.2f',c(TRUE,FALSE,NA))"#,
        error: false,
        stdout: include_str!("raw/sprintf_logical_float.stdout"),
        stderr: include_str!("raw/sprintf_logical_float.stderr"),
    });
}

#[test]
fn sprintf_integer_float() {
    check(Row {
        id: "sprintf_integer_float",
        source: r#"sprintf('%.2f',c(1L,-2L,NA_integer_))"#,
        error: false,
        stdout: include_str!("raw/sprintf_integer_float.stdout"),
        stderr: include_str!("raw/sprintf_integer_float.stderr"),
    });
}

#[test]
fn sprintf_character_float_error() {
    check(Row {
        id: "sprintf_character_float_error",
        source: r#"sprintf('%f','1.2')"#,
        error: true,
        stdout: include_str!("raw/sprintf_character_float_error.stdout"),
        stderr: include_str!("raw/sprintf_character_float_error.stderr"),
    });
}

#[test]
fn sprintf_character_integer_error() {
    check(Row {
        id: "sprintf_character_integer_error",
        source: r#"sprintf('%d','1')"#,
        error: true,
        stdout: include_str!("raw/sprintf_character_integer_error.stdout"),
        stderr: include_str!("raw/sprintf_character_integer_error.stderr"),
    });
}

#[test]
fn sprintf_fixed() {
    check(Row {
        id: "sprintf_fixed",
        source: r#"sprintf(c('%f','%.0f','%.2f','%#.0f','%+010.2f','%-10.2f'),-12.125)"#,
        error: false,
        stdout: include_str!("raw/sprintf_fixed.stdout"),
        stderr: include_str!("raw/sprintf_fixed.stderr"),
    });
}

#[test]
fn sprintf_round_even() {
    check(Row {
        id: "sprintf_round_even",
        source: r#"sprintf('%.0f',c(0.5,1.5,2.5,-0.5,-1.5,-2.5))"#,
        error: false,
        stdout: include_str!("raw/sprintf_round_even.stdout"),
        stderr: include_str!("raw/sprintf_round_even.stderr"),
    });
}

#[test]
fn sprintf_exponent() {
    check(Row {
        id: "sprintf_exponent",
        source: r#"sprintf(c('%e','%E','%.0e','%.3E'),c(1,1e-100,1e100,0))"#,
        error: false,
        stdout: include_str!("raw/sprintf_exponent.stdout"),
        stderr: include_str!("raw/sprintf_exponent.stderr"),
    });
}

#[test]
fn sprintf_general() {
    check(Row {
        id: "sprintf_general",
        source: r#"sprintf(c('%g','%.3g','%#.3g','%G'),c(123456,0.0000123456,12,1e100))"#,
        error: false,
        stdout: include_str!("raw/sprintf_general.stdout"),
        stderr: include_str!("raw/sprintf_general.stderr"),
    });
}
