#[derive(Debug)]
pub(super) enum Json {
    Array(Vec<Json>),
    Text(String),
    Null,
}
pub(super) fn parse(input: &str) -> Result<Json, String> {
    let mut bytes = input.as_bytes();
    let result = value(&mut bytes)?;
    if !bytes.is_empty() {
        return Err("trailing JSON".into());
    }
    Ok(result)
}
fn value(bytes: &mut &[u8]) -> Result<Json, String> {
    match bytes.split_first() {
        Some((b'[', rest)) => {
            *bytes = rest;
            let mut items = Vec::new();
            if let Some(rest) = bytes.strip_prefix(b"]") {
                *bytes = rest;
                return Ok(Json::Array(items));
            }
            loop {
                items.push(value(bytes)?);
                match bytes.split_first() {
                    Some((b',', rest)) => *bytes = rest,
                    Some((b']', rest)) => {
                        *bytes = rest;
                        return Ok(Json::Array(items));
                    }
                    _ => return Err("JSON separator".into()),
                }
            }
        }
        Some((b'"', rest)) => {
            let end = rest.iter().position(|b| *b == b'"').ok_or("JSON quote")?;
            let (text, remaining) = rest.split_at(end);
            if text.contains(&b'\\') {
                return Err("escaped fixture JSON text".into());
            }
            *bytes = remaining.get(1..).ok_or("JSON closing quote")?;
            Ok(Json::Text(
                std::str::from_utf8(text).map_err(|e| e.to_string())?.into(),
            ))
        }
        Some((b'n', _)) if bytes.starts_with(b"null") => {
            *bytes = bytes.get(4..).ok_or("JSON null")?;
            Ok(Json::Null)
        }
        _ => Err("fixture JSON value".into()),
    }
}
pub(super) fn hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) {
        return Err("odd hex length".into());
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
            u8::from_str_radix(text, 16).map_err(|e| e.to_string())
        })
        .collect()
}
pub(super) fn text(value: &Json) -> Result<&str, String> {
    match value {
        Json::Text(value) => Ok(value),
        Json::Null | Json::Array(_) => Err("JSON text".into()),
    }
}
pub(super) fn array(value: &Json) -> Result<&[Json], String> {
    match value {
        Json::Array(value) => Ok(value),
        Json::Null | Json::Text(_) => Err("JSON array".into()),
    }
}
