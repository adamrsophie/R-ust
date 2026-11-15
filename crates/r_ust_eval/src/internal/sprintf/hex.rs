use super::{padding, spec::Spec};

pub(super) fn render(spec: &Spec, value: f64) -> Result<String, String> {
    let bits = value.abs().to_bits();
    let raw_exponent =
        i32::try_from((bits >> 52) & 0x7ff).map_err(|_| "binary64 exponent".to_owned())?;
    let mut fraction = bits & ((1u64 << 52) - 1);
    let exponent = if raw_exponent != 0 {
        fraction |= 1u64 << 52;
        raw_exponent - 1023
    } else if fraction == 0 {
        0
    } else {
        let shift = fraction.leading_zeros() - 11;
        fraction <<= shift;
        -1022 - i32::try_from(shift).map_err(|_| "binary64 shift".to_owned())?
    };
    let precision = spec.precision.unwrap_or(13);
    if spec.precision.is_some() {
        let fixed = 5
            + exponent.unsigned_abs().to_string().len()
            + usize::from(precision > 0 || spec.alternate)
            + spec.sign(value.is_sign_negative()).len();
        padding::limit(precision.saturating_add(fixed).max(spec.width))?;
    }
    let kept = precision.min(13);
    let drop = 4 * (13 - kept);
    let rounded = if drop == 0 {
        fraction
    } else {
        let divisor = 1u64 << drop;
        let quotient = fraction / divisor;
        let remainder = fraction % divisor;
        quotient
            + u64::from(remainder > divisor / 2 || (remainder == divisor / 2 && quotient % 2 == 1))
    };
    let digits = format!("{rounded:0width$x}", width = kept + 1);
    let (head, tail) = digits.split_at(1);
    let tail = if spec.precision.is_none() {
        tail.trim_end_matches('0').to_owned()
    } else {
        format!("{tail}{}", "0".repeat(precision.saturating_sub(13)))
    };
    let dot = if !tail.is_empty() || spec.alternate {
        "."
    } else {
        ""
    };
    let mut body = format!("{head}{dot}{tail}p{exponent:+}");
    let mut prefix = format!("{}0x", spec.sign(value.is_sign_negative()));
    if spec.conversion == 'A' {
        prefix.make_ascii_uppercase();
        body.make_ascii_uppercase();
    }
    padding::pad(spec, (&prefix, &body), true)
}
