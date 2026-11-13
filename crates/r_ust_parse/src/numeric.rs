// Numeric conversion adapted from R 4.6.0, src/main/util.c, R_strtod5.
// Copyright (C) 1997-2025 The R Core Team.
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// SPDX-License-Identifier: GPL-2.0-or-later

mod decimal;
mod hex;

/// Decodes an R numeric string prefix and borrows the unconsumed suffix.
/// Only ASCII C whitespace is skipped; callers decide whether a suffix is valid.
/// Arithmetic matches the ARM binary64 R profile, not wider-LDOUBLE builds.
pub fn parse_numeric_prefix(mut input: &str) -> Option<(f64, &str)> {
    while matches!(
        input.as_bytes().first(),
        Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
    ) {
        input = &input[1..];
    }
    let negative = input.starts_with('-');
    if negative || input.starts_with('+') {
        input = &input[1..];
    }
    for (name, value) in [
        ("NaN", f64::NAN),
        ("Infinity", f64::INFINITY),
        ("Inf", f64::INFINITY),
    ] {
        if input
            .get(..name.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
        {
            return Some((if negative { -value } else { value }, &input[name.len()..]));
        }
    }
    let (value, suffix) = if input.len() > 2 && (input.starts_with("0x") || input.starts_with("0X"))
    {
        hex::decode(&input[2..])?
    } else {
        decimal::decode(input)?
    };
    Some((if negative { -value } else { value }, suffix))
}

fn exponent(mut input: &str) -> Option<(i64, &str)> {
    let negative = input.starts_with('-');
    if negative || input.starts_with('+') {
        input = &input[1..];
    }
    let start = input.len();
    let mut value = 0;
    while let Some(digit @ b'0'..=b'9') = input.as_bytes().first() {
        if value < 9999 {
            value = value * 10 + i64::from(digit - b'0');
        }
        input = &input[1..];
    }
    (input.len() != start).then_some((if negative { -value } else { value }, input))
}
