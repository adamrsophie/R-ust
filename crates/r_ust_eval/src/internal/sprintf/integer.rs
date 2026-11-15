use super::{padding, spec::Spec};

pub(super) fn render(spec: &Spec, value: Option<i32>) -> Result<String, String> {
    let Some(value) = value else {
        return padding::string(spec, "NA");
    };
    let unsigned = u32::from_ne_bytes(value.to_ne_bytes());
    let mut digits = match spec.conversion {
        'o' => format!("{unsigned:o}"),
        'x' => format!("{unsigned:x}"),
        'X' => format!("{unsigned:X}"),
        _ => value.unsigned_abs().to_string(),
    };
    if value == 0 && spec.precision == Some(0) {
        digits.clear();
    }
    let zeros = spec.precision.unwrap_or(0).saturating_sub(digits.len());
    let prefix = match spec.conversion {
        'd' | 'i' => spec.sign(value < 0),
        'o' if spec.alternate && zeros == 0 && !digits.starts_with('0') => "0",
        'x' if spec.alternate && value != 0 => "0x",
        'X' if spec.alternate && value != 0 => "0X",
        _ => "",
    };
    padding::limit(
        prefix
            .len()
            .saturating_add(zeros)
            .saturating_add(digits.len())
            .max(spec.width),
    )?;
    digits = format!("{}{digits}", "0".repeat(zeros));
    padding::pad(spec, (prefix, &digits), spec.precision.is_none())
}
