// Decimal conversion adapted from R 4.6.0, src/main/util.c, R_strtod5.
// Copyright (C) 1997-2025 The R Core Team.
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// SPDX-License-Identifier: GPL-2.0-or-later

pub(super) fn decode(mut input: &str) -> Option<(f64, &str)> {
    let (mut value, whole) = digits(&mut input, 0.0);
    let mut fractional = 0;
    if input.starts_with('.') {
        input = &input[1..];
        (value, fractional) = digits(&mut input, value);
    }
    if whole + fractional == 0 {
        return None;
    }
    let mut exponent = -fractional;
    if matches!(input.as_bytes().first(), Some(b'e' | b'E')) {
        let (explicit, suffix) = super::exponent(&input[1..])?;
        exponent += explicit;
        input = suffix;
    }
    Some((scale(value, exponent, whole + fractional), input))
}

fn digits(input: &mut &str, mut value: f64) -> (f64, i64) {
    let mut count = 0;
    // shortcut: binary64 LDOUBLE and fused accumulation match arm64 R; wider LDOUBLE needs wider arithmetic.
    while let Some(digit @ b'0'..=b'9') = input.as_bytes().first() {
        value = value.mul_add(10.0, f64::from(digit - b'0'));
        count += 1;
        *input = &input[1..];
    }
    (value, count)
}

fn scale(mut value: f64, mut exponent: i64, digits: i64) -> f64 {
    if exponent + digits < -300 {
        for _ in 0..digits {
            value /= 10.0;
        }
        exponent += digits;
    }
    if exponent >= 0 && value == 0.0 {
        return value;
    }
    let reciprocal = exponent < -307;
    let mut remaining = exponent.unsigned_abs();
    let mut power = 10.0;
    let mut factor = 1.0;
    while remaining != 0 {
        if remaining & 1 != 0 {
            if reciprocal {
                factor /= power;
            } else {
                factor *= power;
            }
        }
        remaining >>= 1;
        power *= power;
    }
    if reciprocal || exponent >= 0 {
        value * factor
    } else {
        value / factor
    }
}
