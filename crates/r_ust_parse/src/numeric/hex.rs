// Hexadecimal conversion adapted from R 4.6.0, src/main/util.c, R_strtod5.
// Copyright (C) 1997-2025 The R Core Team.
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka.
// SPDX-License-Identifier: GPL-2.0-or-later

pub(super) fn decode(mut input: &str) -> Option<(f64, &str)> {
    let mut fractional_bits = -1_i64;
    // shortcut: accumulation matches R with 64-bit LDOUBLE; use wider arithmetic for wider-LDOUBLE hosts.
    let mut value = 0.0;
    while let Some(byte) = input.as_bytes().first() {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            b'.' => {
                fractional_bits = 0;
                input = &input[1..];
                continue;
            }
            _ => break,
        };
        value = 16.0 * value + f64::from(digit);
        if fractional_bits >= 0 {
            fractional_bits += 4;
        }
        input = &input[1..];
    }
    let mut exponent = 0;
    if matches!(input.as_bytes().first(), Some(b'p' | b'P')) {
        (exponent, input) = super::exponent(&input[1..])?;
    }
    if value != 0.0 {
        if fractional_bits > 0 {
            if exponent - fractional_bits < -122 {
                value /= binary_factor(fractional_bits.unsigned_abs());
            } else {
                exponent -= fractional_bits;
            }
        }
        let factor = binary_factor(exponent.unsigned_abs());
        value = if exponent < 0 {
            value / factor
        } else {
            value * factor
        };
    }
    Some((value, input))
}

fn binary_factor(exponent: u64) -> f64 {
    if exponent >= 1024 {
        f64::INFINITY
    } else {
        f64::from_bits((exponent + 1023) << 52)
    }
}
