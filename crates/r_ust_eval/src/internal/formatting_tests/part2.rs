use super::{check, Row};

#[test]
fn sprintf_integer_flags() {
    check(Row {
        id: "sprintf_integer_flags",
        source: r#"sprintf(c('%+d','% d','%08d','%-8d','%.4d','%8.4d'),-12L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_integer_flags.stdout"),
        stderr: include_str!("raw/sprintf_integer_flags.stderr"),
    });
}

#[test]
fn sprintf_unsigned_hex() {
    check(Row {
        id: "sprintf_unsigned_hex",
        source: r#"sprintf(c('%x','%X','%o','%#x','%#X','%#o'),-1L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_unsigned_hex.stdout"),
        stderr: include_str!("raw/sprintf_unsigned_hex.stderr"),
    });
}

#[test]
fn sprintf_zero_precision() {
    check(Row {
        id: "sprintf_zero_precision",
        source: r#"sprintf(c('%.0d','%#.0o','%#.0x','%#.0X'),0L)"#,
        error: false,
        stdout: include_str!("raw/sprintf_zero_precision.stdout"),
        stderr: include_str!("raw/sprintf_zero_precision.stderr"),
    });
}

#[test]
fn sprintf_integer_na_flags() {
    check(Row {
        id: "sprintf_integer_na_flags",
        source: r#"sprintf(c('%08d','%+d','%.1d','%8.1d'),NA_integer_)"#,
        error: false,
        stdout: include_str!("raw/sprintf_integer_na_flags.stdout"),
        stderr: include_str!("raw/sprintf_integer_na_flags.stderr"),
    });
}

#[test]
fn sprintf_integral_double() {
    check(Row {
        id: "sprintf_integral_double",
        source: r#"sprintf('%d',c(1,2,NA_real_))"#,
        error: false,
        stdout: include_str!("raw/sprintf_integral_double.stdout"),
        stderr: include_str!("raw/sprintf_integral_double.stderr"),
    });
}

#[test]
fn sprintf_fractional_double_error() {
    check(Row {
        id: "sprintf_fractional_double_error",
        source: r#"sprintf('%d',c(1,1.5))"#,
        error: true,
        stdout: include_str!("raw/sprintf_fractional_double_error.stdout"),
        stderr: include_str!("raw/sprintf_fractional_double_error.stderr"),
    });
}

#[test]
fn sprintf_nan_integer_error() {
    check(Row {
        id: "sprintf_nan_integer_error",
        source: r#"sprintf('%d',NaN)"#,
        error: true,
        stdout: include_str!("raw/sprintf_nan_integer_error.stdout"),
        stderr: include_str!("raw/sprintf_nan_integer_error.stderr"),
    });
}

#[test]
fn sprintf_overflow_integer_error() {
    check(Row {
        id: "sprintf_overflow_integer_error",
        source: r#"sprintf('%d',2147483648)"#,
        error: true,
        stdout: include_str!("raw/sprintf_overflow_integer_error.stdout"),
        stderr: include_str!("raw/sprintf_overflow_integer_error.stderr"),
    });
}
