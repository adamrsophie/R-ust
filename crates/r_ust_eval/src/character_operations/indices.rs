// Preserve the existing native-width truncation/saturation boundary. R integer
// coercion and its warnings remain a separate unsupported substring boundary.
// Callers reject nonfinite inputs; the length caller also rejects negative values.
pub(super) fn saturating_index(value: f64) -> isize {
    let magnitude = magnitude(value).and_then(|value| isize::try_from(value).ok());
    if value.is_sign_negative() {
        magnitude.map_or(isize::MIN, |value| -value)
    } else {
        magnitude.unwrap_or(isize::MAX)
    }
}

pub(super) fn saturating_length(value: f64) -> usize {
    magnitude(value)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(usize::MAX)
}

fn magnitude(value: f64) -> Option<u64> {
    let bits = value.to_bits();
    let biased_exponent = (bits >> 52) & 0x7ff;
    if biased_exponent < 1023 {
        return Some(0);
    }
    let exponent = u32::try_from(biased_exponent - 1023).ok()?;
    if exponent >= u64::BITS {
        return None;
    }
    let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    if exponent <= 52 {
        Some(significand >> (52 - exponent))
    } else {
        // The exponent bound also bounds the highest set bit after shifting.
        significand.checked_shl(exponent - 52)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_values_truncate_and_saturate_at_native_bounds() {
        for (input, expected) in [
            (2.9, 2),
            (-2.9, -2),
            (0.0, 0),
            (-0.0, 0),
            (f64::MIN_POSITIVE, 0),
            (-f64::MIN_POSITIVE, 0),
            (f64::from_bits(1), 0),
            (-f64::from_bits(1), 0),
        ] {
            assert_eq!(saturating_index(input), expected);
        }
        let signed_bound = f64::from_bits((1023 + u64::from(isize::BITS - 1)) << 52);
        let unsigned_bound = f64::from_bits((1023 + u64::from(usize::BITS)) << 52);
        assert_eq!(saturating_index(signed_bound), isize::MAX);
        assert_eq!(saturating_index(-signed_bound), isize::MIN);
        assert_eq!(saturating_index(f64::MAX), isize::MAX);
        assert_eq!(saturating_index(-f64::MAX), isize::MIN);
        assert_eq!(saturating_length(unsigned_bound), usize::MAX);
        assert_eq!(saturating_length(f64::MAX), usize::MAX);
        assert_eq!(saturating_length(2.9), 2);
        assert_eq!(saturating_length(-0.0), 0);
        assert_eq!(saturating_length(f64::from_bits(1)), 0);
        assert!(saturating_length(f64::from_bits(unsigned_bound.to_bits() - 1)) < usize::MAX);
        assert_eq!(magnitude(18446744073709551616.0), None);
        assert_eq!(
            magnitude(f64::from_bits(18446744073709551616.0_f64.to_bits() - 1)),
            Some(u64::MAX - 2047)
        );
    }
}
