//! Scalar deparse profile: R 4.6.0, digits=15, scipen=0, C decimal mark.
//! `scientific` follows format.c's binary64 branch (native arm64 R).

const POWERS: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];

#[cfg(test)]
#[path = "real_literal_tests.rs"]
mod tests;

pub(super) fn format_real(value: f64, keep_na: bool) -> String {
    if r_ust_sexp::is_na_real(value) {
        return if keep_na { "NA_real_" } else { "NA" }.into();
    }
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Inf"
        } else {
            "Inf"
        }
        .into();
    }
    // EncodeReal0 normalizes both IEEE zero signs before formatting.
    if value == 0.0 {
        return "0".into();
    }
    let (power, significant, widens) = scientific(value.abs());
    let left = power + 1 - i16::from(widens);
    let right = if left < 0 {
        significant + usize::from(left.unsigned_abs())
    } else {
        significant.saturating_sub(usize::from(left.unsigned_abs()))
    };
    let fixed_width = usize::from(left.max(1).unsigned_abs()) + right + usize::from(right != 0);
    let exponent_width = if left > 100 || left <= -99 { 3 } else { 2 };
    let scientific_width = significant + usize::from(significant > 1) + 2 + exponent_width;
    if fixed_width <= scientific_width {
        format!("{value:.right$}")
    } else {
        let precision = significant - 1;
        let encoded = format!("{value:.precision$e}");
        let mut parts = encoded.split('e');
        let mut result = String::new();
        if let Some(mantissa) = parts.next() {
            result.push_str(mantissa);
        }
        if let Some(exponent) = parts.next() {
            result.push('e');
            let magnitude = if let Some(magnitude) = exponent.strip_prefix('-') {
                result.push('-');
                magnitude
            } else {
                result.push('+');
                exponent
            };
            if magnitude.len() < 2 {
                result.push('0');
            }
            result.push_str(magnitude);
        }
        result
    }
}

fn scientific(value: f64) -> (i16, usize, bool) {
    let floor = value.log10().floor();
    let mut power = 0_i16;
    // Binary64 decimal exponents fit in i16; avoid a truncating float cast.
    while f64::from(power) < floor {
        power += 1;
    }
    while f64::from(power) > floor {
        power -= 1;
    }
    power -= 14;
    let mut scaled = if power.unsigned_abs() <= 22 {
        let factor = POWERS[usize::from(power.unsigned_abs())];
        if power >= 0 {
            value / factor
        } else {
            value * factor
        }
    } else if power <= -308 {
        (value * 1e303) / 10_f64.powf(f64::from(power + 303))
    } else {
        value / 10_f64.powf(f64::from(power))
    };
    if scaled < 1e14 {
        scaled *= 10.0;
        power -= 1;
    }
    let mut alpha = scaled.round_ties_even();
    let mut significant = 15;
    for _ in 0..15 {
        alpha /= 10.0;
        if alpha != alpha.floor() {
            break;
        }
        significant -= 1;
    }
    if significant == 0 {
        significant = 1;
        power += 1;
    }
    power += 14;
    let right = (15 - power).clamp(0, 22);
    let fuzz = 0.5 / POWERS[usize::from(right.unsigned_abs())];
    let widens =
        power > 0 && power <= 22 && value < POWERS[usize::from(power.unsigned_abs())] - fuzz;
    (power, significant, widens)
}
