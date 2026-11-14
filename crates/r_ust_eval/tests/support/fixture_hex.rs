pub(crate) fn hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("invalid lowercase hex".into());
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(
                std::str::from_utf8(pair).map_err(|error| error.to_string())?,
                16,
            )
            .map_err(|error| error.to_string())
        })
        .collect()
}

pub(crate) fn unhex(text: &str) -> Result<String, String> {
    String::from_utf8(hex(text)?).map_err(|error| error.to_string())
}
