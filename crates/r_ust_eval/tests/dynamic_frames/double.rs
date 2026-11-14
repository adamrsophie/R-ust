pub(super) fn bits(text: &str) -> Result<u64, String> {
    let (negative, value) = text
        .strip_prefix('-')
        .map_or((false, text), |value| (true, value));
    let sign = u64::from(negative) << 63;
    if value == "Inf" {
        return Ok(sign | 0x7ff0_0000_0000_0000);
    }
    let (significand, exponent) = value
        .strip_prefix("0x")
        .ok_or("hex double prefix")?
        .split_once('p')
        .ok_or("hex double exponent")?;
    if !exponent.starts_with(['+', '-']) {
        return Err("hex double exponent sign".into());
    }
    let exponent: i32 = exponent.parse().map_err(|_| "hex double exponent range")?;
    let (whole, fraction) = significand.split_once('.').unwrap_or((significand, ""));
    if whole.is_empty()
        || whole.len() + fraction.len() > 16
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("hex double significand".into());
    }
    let coefficient = u64::from_str_radix(&format!("{whole}{fraction}"), 16)
        .map_err(|_| "hex double coefficient")?;
    let fraction_bits = i32::try_from(fraction.len())
        .map_err(|_| "hex fraction length")?
        .checked_mul(4)
        .ok_or("hex fraction overflow")?;
    let exponent = exponent
        .checked_sub(fraction_bits)
        .ok_or("hex exponent overflow")?;
    if coefficient == 0 {
        return Ok(sign);
    }
    let width = 64 - coefficient.leading_zeros();
    let top = exponent
        .checked_add(i32::try_from(width).map_err(|_| "hex width")? - 1)
        .ok_or("hex exponent overflow")?;
    if top > 1023 {
        return Err("finite hex double overflow".into());
    }
    let magnitude = if top >= -1022 {
        let normalized = shift(
            coefficient,
            53 - i32::try_from(width).map_err(|_| "hex width")?,
        )?;
        (u64::try_from(top + 1023).map_err(|_| "hex exponent")? << 52)
            | (normalized & 0x000f_ffff_ffff_ffff)
    } else {
        shift(
            coefficient,
            exponent.checked_add(1074).ok_or("hex subnormal exponent")?,
        )?
    };
    Ok(sign | magnitude)
}

fn shift(value: u64, places: i32) -> Result<u64, String> {
    if places >= 0 {
        let places = u32::try_from(places).map_err(|_| "hex shift")?;
        let shifted = value.checked_shl(places).ok_or("hex shift overflow")?;
        if shifted.checked_shr(places) != Some(value) {
            return Err("hex mantissa overflow".into());
        }
        Ok(shifted)
    } else {
        let places = places.unsigned_abs();
        let shifted = value.checked_shr(places).ok_or("hex shift underflow")?;
        if shifted.checked_shl(places) != Some(value) {
            return Err("inexact hex double".into());
        }
        Ok(shifted)
    }
}

#[test]
fn hexadecimal_doubles_preserve_ieee_bits() {
    for (text, expected) in [
        ("0x0p+0", 0),
        ("-0x0p+0", 0x8000_0000_0000_0000),
        ("0x1p+0", 0x3ff0_0000_0000_0000),
        ("0x1.8p+2", 0x4018_0000_0000_0000),
        ("0x1.999999999999ap-4", 0x3fb9_9999_9999_999a),
        ("0x1p-1074", 1),
        ("0x0.0000000000001p-1022", 1),
        ("0x1p-1022", 0x0010_0000_0000_0000),
        ("0x1.fffffffffffffp+1023", 0x7fef_ffff_ffff_ffff),
        ("Inf", 0x7ff0_0000_0000_0000),
        ("-Inf", 0xfff0_0000_0000_0000),
    ] {
        assert_eq!(bits(text).unwrap(), expected, "{text}");
    }
    for text in [
        "1.0",
        "0x1p0",
        "0x1p+1024",
        "0x1p-1075",
        "0x1.00000000000001p+0",
        "0x1p-2147483648",
        "0xzzp+0",
    ] {
        assert!(bits(text).is_err(), "{text}");
    }
}
