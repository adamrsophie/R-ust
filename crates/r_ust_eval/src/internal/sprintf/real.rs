use super::{hex, padding, spec::Spec};

pub(super) fn render(spec: &Spec, value: f64) -> Result<String, String> {
    if !value.is_finite() {
        let text = if r_ust_sexp::is_na_real(value) {
            if spec.space {
                " NA"
            } else {
                "NA"
            }
        } else if value.is_nan() {
            if spec.space {
                " NaN"
            } else {
                "NaN"
            }
        } else if value.is_sign_negative() {
            "-Inf"
        } else if spec.plus {
            "+Inf"
        } else if spec.space {
            " Inf"
        } else {
            "Inf"
        };
        let mut text_spec = spec.clone();
        text_spec.precision = None;
        return padding::string(&text_spec, text);
    }
    if matches!(spec.conversion, 'a' | 'A') {
        return hex::render(spec, value);
    }
    let precision = spec.precision.unwrap_or(6);
    let magnitude = value.abs();
    if precision > padding::MAX_LENGTH {
        large_precision(spec, value)?;
    }
    let mut body = match spec.conversion {
        'e' | 'E' => scientific(magnitude, precision, spec.alternate)?,
        'g' | 'G' => general(magnitude, precision.max(1), spec.alternate)?,
        _ => {
            let mut text = format!("{magnitude:.precision$}");
            if spec.alternate && precision == 0 {
                text.push('.');
            }
            text
        }
    };
    if matches!(spec.conversion, 'E' | 'G') {
        body.make_ascii_uppercase();
    }
    padding::pad(spec, (spec.sign(value.is_sign_negative()), &body), true)
}

fn scientific(value: f64, precision: usize, alternate: bool) -> Result<String, String> {
    let encoded = format!("{value:.precision$e}");
    let Some((mantissa, exponent)) = encoded.split_once('e') else {
        return Err("invalid decimal exponent".into());
    };
    let exponent = exponent
        .parse::<i32>()
        .map_err(|_| "invalid decimal exponent".to_owned())?;
    let dot = if alternate && precision == 0 { "." } else { "" };
    let sign = if exponent < 0 { '-' } else { '+' };
    Ok(format!(
        "{mantissa}{dot}e{sign}{:02}",
        exponent.unsigned_abs()
    ))
}

fn general(value: f64, precision: usize, alternate: bool) -> Result<String, String> {
    let precision = if alternate {
        precision
    } else {
        precision.min(1074)
    };
    let scientific = scientific(value, precision - 1, alternate)?;
    let Some((mantissa, exponent)) = scientific.split_once('e') else {
        return Err("invalid decimal exponent".into());
    };
    let power = exponent
        .parse::<i32>()
        .map_err(|_| "invalid decimal exponent".to_owned())?;
    let threshold =
        i32::try_from(precision).map_err(|_| "format precision is too large".to_owned())?;
    if power < -4 || power >= threshold {
        let mantissa = if alternate { mantissa } else { trim(mantissa) };
        return Ok(format!("{mantissa}e{exponent}"));
    }
    let decimals = usize::try_from(threshold - power - 1)
        .map_err(|_| "invalid decimal precision".to_owned())?;
    let mut text = format!("{value:.decimals$}");
    if alternate && decimals == 0 {
        text.push('.');
    }
    Ok(if alternate { text } else { trim(&text).into() })
}

fn trim(value: &str) -> &str {
    if value.contains('.') {
        value.trim_end_matches('0').trim_end_matches('.')
    } else {
        value
    }
}

fn large_precision(spec: &Spec, value: f64) -> Result<(), String> {
    if matches!(spec.conversion, 'g' | 'G') && !spec.alternate {
        return Ok(());
    }
    let precision = spec.precision.unwrap_or(6);
    let encoded = format!("{:e}", value.abs());
    let (_, power) = encoded
        .split_once('e')
        .ok_or_else(|| "invalid decimal exponent".to_owned())?;
    let exponent = power
        .parse::<i32>()
        .map_err(|_| "invalid decimal exponent".to_owned())?;
    let exponent_digits = exponent.unsigned_abs().to_string().len().max(2);
    let length = match spec.conversion {
        'e' | 'E' => precision.saturating_add(4 + exponent_digits),
        'g' | 'G' if exponent < -4 => precision.saturating_add(3 + exponent_digits),
        'g' | 'G' => precision.saturating_add(
            1 + usize::try_from((-exponent).max(0)).map_err(|_| "decimal exponent".to_owned())?,
        ),
        _ => precision.saturating_add(1 + format!("{:.0}", value.abs().floor()).len()),
    };
    padding::limit(
        length
            .saturating_add(spec.sign(value.is_sign_negative()).len())
            .max(spec.width),
    )
}
