use super::LanguagePrintError;
pub(crate) fn check_text(text: &str) -> Result<(), LanguagePrintError> {
    if text.contains('\0') {
        Err(LanguagePrintError::InvalidText)
    }
    // shortcut: ASCII text only, add R encoding provenance before accepting non-ASCII.
    else if !text.is_ascii() {
        Err(LanguagePrintError::UnsupportedText)
    } else {
        Ok(())
    }
}
pub(crate) fn valid_name(text: &str) -> bool {
    let mut bytes = text.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'.')
        && !(first == b'.' && bytes.clone().next().is_some_and(|b| b.is_ascii_digit()))
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
        && !matches!(
            text,
            "NULL"
                | "NA"
                | "TRUE"
                | "FALSE"
                | "Inf"
                | "NaN"
                | "NA_integer_"
                | "NA_real_"
                | "NA_character_"
                | "NA_complex_"
                | "function"
                | "while"
                | "repeat"
                | "for"
                | "if"
                | "in"
                | "else"
                | "next"
                | "break"
        )
}
pub(super) fn name(text: &str) -> Result<String, LanguagePrintError> {
    check_text(text)?;
    Ok(if text.is_empty() || valid_name(text) {
        text.to_owned()
    } else {
        crate::encode_string::encode(text, Some('`'))
    })
}
