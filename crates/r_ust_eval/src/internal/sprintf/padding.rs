use super::spec::Spec;

pub(super) const MAX_LENGTH: usize = 8192;

pub(super) fn limit(length: usize) -> Result<(), String> {
    if length > MAX_LENGTH {
        Err(format!(
            "required resulting string length {length} is greater than maximal {MAX_LENGTH}"
        ))
    } else {
        Ok(())
    }
}

pub(super) fn pad(spec: &Spec, parts: (&str, &str), numeric_zero: bool) -> Result<String, String> {
    let (prefix, body) = parts;
    let length = prefix.len().saturating_add(body.len());
    limit(length.max(spec.width))?;
    let count = spec.width.saturating_sub(length);
    let padding = if spec.zero && !spec.left && numeric_zero {
        '0'
    } else {
        ' '
    };
    let fill = padding.to_string().repeat(count);
    Ok(if spec.left {
        format!("{prefix}{body}{fill}")
    } else if padding == '0' {
        format!("{prefix}{fill}{body}")
    } else {
        format!("{fill}{prefix}{body}")
    })
}

pub(super) fn string(spec: &Spec, value: &str) -> Result<String, String> {
    let end = spec.precision.unwrap_or(value.len()).min(value.len());
    let Some(value) = value.get(..end) else {
        return Err(
            "format precision splits a UTF-8 character in the current string representation".into(),
        );
    };
    pad(spec, ("", value), true)
}
