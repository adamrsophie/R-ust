const QUIET: u64 = 0x0008_0000_0000_0000;
const DEFAULT_NAN: f64 = f64::from_bits(0x7ff8_0000_0000_0000);

fn nan(left: f64, right: f64) -> Option<f64> {
    // R's pinned ARM profile prefers signaling operands, then the left quiet NaN.
    let value = if left.is_nan() && left.to_bits() & QUIET == 0 {
        left
    } else if right.is_nan() && right.to_bits() & QUIET == 0 {
        right
    } else if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else {
        return None;
    };
    Some(f64::from_bits(value.to_bits() | QUIET))
}

pub(super) fn add(left: f64, right: f64) -> f64 {
    nan(left, right).unwrap_or_else(|| {
        let result = left + right;
        if result.is_nan() {
            DEFAULT_NAN
        } else {
            result
        }
    })
}

pub(super) fn multiply(left: f64, right: f64) -> f64 {
    nan(left, right).unwrap_or_else(|| {
        let result = left * right;
        if result.is_nan() {
            DEFAULT_NAN
        } else {
            result
        }
    })
}

pub(super) fn integer_to_real(value: i64) -> f64 {
    let bytes = value.unsigned_abs().to_le_bytes();
    let low = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let magnitude = f64::from(high) * 4_294_967_296.0 + f64::from(low);
    if value < 0 {
        -magnitude
    } else {
        magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::integer_to_real;

    #[test]
    fn integer_conversion_matches_independent_r_subtotals() {
        let rows = include_str!("../../tests/fixtures/summary-integer-conversion-r-4.6.0.tsv");
        let mut count = 0;
        for row in rows.lines().skip(1) {
            let fields: Vec<_> = row.split('\t').collect();
            let sign: i64 = fields[0].parse().unwrap();
            let repetitions: i64 = fields[1].parse().unwrap();
            let offset: i64 = fields[2].parse().unwrap();
            let subtotal = sign * (i64::from(i32::MAX) * repetitions + offset);
            assert_eq!(
                integer_to_real(subtotal).to_bits(),
                u64::from_str_radix(fields[3], 16).unwrap(),
                "{row}"
            );
            count += 1;
        }
        assert_eq!(count, 24);
    }

    #[test]
    fn integer_conversion_rounds_ieee_halfway_and_signed_edges() {
        for (integer, bits) in [
            (0, 0x0000_0000_0000_0000),
            (1, 0x3ff0_0000_0000_0000),
            (-1, 0xbff0_0000_0000_0000),
            (9_007_199_254_740_991, 0x433f_ffff_ffff_ffff),
            (9_007_199_254_740_993, 0x4340_0000_0000_0000),
            (9_007_199_254_740_995, 0x4340_0000_0000_0002),
            (-9_007_199_254_740_995, 0xc340_0000_0000_0002),
            (i64::MAX, 0x43e0_0000_0000_0000),
            (i64::MIN, 0xc3e0_0000_0000_0000),
        ] {
            assert_eq!(integer_to_real(integer).to_bits(), bits, "{integer}");
        }
    }
}
