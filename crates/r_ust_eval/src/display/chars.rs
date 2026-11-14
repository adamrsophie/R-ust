use crate::EvalError;

pub(super) fn format(value: &Option<String>) -> Result<String, EvalError> {
    let Some(value) = value else {
        return Ok("<CHARSXP: NA>".into());
    };
    if value.contains('\0') {
        return Err(EvalError::NotImplemented("CHARSXP with embedded NUL"));
    }
    // shortcut: translation follows the pinned C locale, extend before supporting another locale.
    let native = value
        .chars()
        .map(|character| {
            let code = u32::from(character);
            if character.is_ascii() {
                character.to_string()
            } else if code < 65536 {
                format!("<U+{code:04X}>")
            } else {
                format!("<U+{code:08X}>")
            }
        })
        .collect::<String>();
    Ok(format!(
        "<CHARSXP: {}>",
        crate::encode_string::encode(&native, Some('"'))
    ))
}
