use crate::{value_len, EvalError, Sexp};

pub(super) fn convert(x: &Sexp, base: &Sexp) -> Result<Sexp, EvalError> {
    let Sexp::Integer(bases) = base else {
        return Err(EvalError::Domain("invalid 'base' argument".into()));
    };
    let Some(Some(base)) = bases.first() else {
        return Err(EvalError::Domain("invalid 'base' argument".into()));
    };
    if *base != 0 && !(2..=36).contains(base) {
        return Err(EvalError::Domain("invalid 'base' argument".into()));
    }
    if value_len(x) == 0 {
        return Ok(Sexp::Integer(Vec::new()));
    }
    let Sexp::String(values) = x else {
        return Err(EvalError::Type("strtoi requires character storage".into()));
    };
    let radix =
        u32::try_from(*base).map_err(|_| EvalError::Domain("invalid 'base' argument".into()))?;
    Ok(Sexp::Integer(
        values
            .iter()
            .map(|value| value.as_deref().and_then(|text| parse(text, radix)))
            .collect(),
    ))
}

fn parse(text: &str, radix: u32) -> Option<i32> {
    // R uses ASCII strtol on stored bytes; the runtime currently stores UTF-8 only.
    let text = text.trim_start_matches([' ', '\t', '\n', '\r', '\u{000b}', '\u{000c}']);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let hexadecimal = digits.starts_with("0x") || digits.starts_with("0X");
    let radix = match radix {
        0 if hexadecimal => 16,
        0 if digits.starts_with('0') => 8,
        0 => 10,
        radix => radix,
    };
    let digits = if radix == 16 && hexadecimal {
        &digits[2..]
    } else {
        digits
    };
    if digits.is_empty() {
        return None;
    }
    let magnitude = digits.bytes().try_fold(0_i32, |value, byte| {
        let digit = char::from(byte).to_digit(radix)?;
        value
            .checked_mul(i32::try_from(radix).ok()?)?
            .checked_add(i32::try_from(digit).ok()?)
    })?;
    // R reserves INT_MIN for NA, so both signs have the same usable magnitude.
    Some(if negative { -magnitude } else { magnitude })
}
