use super::{check, Row};

#[test]
fn sprintf_general_boundaries() {
    check(Row {
        id: "sprintf_general_boundaries",
        source: r#"sprintf('%.4g',c(0.00009999,0.00001,9999,99999,9999.9))"#,
        error: false,
        stdout: include_str!("raw/sprintf_general_boundaries.stdout"),
        stderr: include_str!("raw/sprintf_general_boundaries.stderr"),
    });
}

#[test]
fn sprintf_hex_float() {
    check(Row {
        id: "sprintf_hex_float",
        source: r#"sprintf(c('%a','%A','%.2a','%#.0a'),c(1.5,0.1,1.999,1))"#,
        error: false,
        stdout: include_str!("raw/sprintf_hex_float.stdout"),
        stderr: include_str!("raw/sprintf_hex_float.stderr"),
    });
}

#[test]
fn sprintf_subnormal() {
    check(Row {
        id: "sprintf_subnormal",
        source: r#"sprintf(c('%a','%.13a','%.17g'),c(5e-324,2.2250738585072014e-308,0.1))"#,
        error: false,
        stdout: include_str!("raw/sprintf_subnormal.stdout"),
        stderr: include_str!("raw/sprintf_subnormal.stderr"),
    });
}

#[test]
fn sprintf_negative_zero() {
    check(Row {
        id: "sprintf_negative_zero",
        source: r#"sprintf(c('%f','%e','%g','%a'),-0.0)"#,
        error: false,
        stdout: include_str!("raw/sprintf_negative_zero.stdout"),
        stderr: include_str!("raw/sprintf_negative_zero.stderr"),
    });
}

#[test]
fn sprintf_nonfinite() {
    check(Row {
        id: "sprintf_nonfinite",
        source: r#"sprintf('%+10.2f',c(NA_real_,NaN,Inf,-Inf))"#,
        error: false,
        stdout: include_str!("raw/sprintf_nonfinite.stdout"),
        stderr: include_str!("raw/sprintf_nonfinite.stderr"),
    });
}

#[test]
fn sprintf_nonfinite_space() {
    check(Row {
        id: "sprintf_nonfinite_space",
        source: r#"sprintf('% 10.2f',c(NA_real_,NaN,Inf,-Inf))"#,
        error: false,
        stdout: include_str!("raw/sprintf_nonfinite_space.stdout"),
        stderr: include_str!("raw/sprintf_nonfinite_space.stderr"),
    });
}

#[test]
fn sprintf_string_flags() {
    check(Row {
        id: "sprintf_string_flags",
        source: r#"sprintf(c('%s','%8s','%-8s','%.3s','%8.3s','%08s'),'abcdef')"#,
        error: false,
        stdout: include_str!("raw/sprintf_string_flags.stdout"),
        stderr: include_str!("raw/sprintf_string_flags.stderr"),
    });
}

#[test]
fn sprintf_string_na() {
    check(Row {
        id: "sprintf_string_na",
        source: r#"sprintf(c('%s','%.1s','%5s'),NA_character_)"#,
        error: false,
        stdout: include_str!("raw/sprintf_string_na.stdout"),
        stderr: include_str!("raw/sprintf_string_na.stderr"),
    });
}
