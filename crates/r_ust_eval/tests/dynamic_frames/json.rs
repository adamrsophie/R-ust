use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) enum Json {
    Object(BTreeMap<String, Json>),
    Array(Vec<Json>),
    Text(String),
    Integer(i32),
    Null,
}

impl Json {
    pub(super) fn get(&self, name: &str) -> Result<&Self, String> {
        match self {
            Self::Object(fields) => fields
                .get(name)
                .ok_or_else(|| format!("missing JSON {name}")),
            _ => Err("expected JSON object".into()),
        }
    }

    pub(super) fn text(&self) -> Result<&str, String> {
        match self {
            Self::Text(text) => Ok(text),
            _ => Err("expected JSON text".into()),
        }
    }

    pub(super) fn array(&self) -> Result<&[Self], String> {
        match self {
            Self::Array(items) => Ok(items),
            _ => Err("expected JSON array".into()),
        }
    }

    pub(super) fn integer(&self) -> Result<i32, String> {
        match self {
            Self::Integer(value) => Ok(*value),
            _ => Err("expected JSON integer".into()),
        }
    }
}

#[path = "../support/fixture_hex.rs"]
mod hex_codec;
pub(super) use hex_codec::{hex, unhex};

pub(super) fn parse(text: &str) -> Result<Json, String> {
    let mut parser = Parser(text.as_bytes());
    let result = parser.value(0)?;
    parser.space();
    if !parser.0.is_empty() {
        return Err("trailing JSON".into());
    }
    Ok(result)
}

struct Parser<'a>(&'a [u8]);

impl Parser<'_> {
    fn space(&mut self) {
        self.0 = self.0.trim_ascii_start();
    }

    fn take(&mut self) -> Result<u8, String> {
        let (byte, rest) = self.0.split_first().ok_or("truncated JSON")?;
        self.0 = rest;
        Ok(*byte)
    }

    fn string(&mut self) -> Result<String, String> {
        if self.take()? != b'"' {
            return Err("JSON quote".into());
        }
        let mut bytes = Vec::new();
        loop {
            match self.take()? {
                b'"' => return String::from_utf8(bytes).map_err(|error| error.to_string()),
                b'\\' => bytes.push(match self.take()? {
                    b'"' => b'"',
                    b'\\' => b'\\',
                    b'/' => b'/',
                    b'b' => 8,
                    b'f' => 12,
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    _ => return Err("unsupported fixture JSON escape".into()),
                }),
                byte if byte < 32 => return Err("JSON control byte".into()),
                byte => bytes.push(byte),
            }
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > 64 {
            return Err("fixture JSON depth".into());
        }
        self.space();
        match self.0.first() {
            Some(b'"') => self.string().map(Json::Text),
            Some(b'{') => {
                self.take()?;
                let mut fields = BTreeMap::new();
                self.space();
                if self.0.first() == Some(&b'}') {
                    self.take()?;
                    return Ok(Json::Object(fields));
                }
                loop {
                    self.space();
                    let key = self.string()?;
                    self.space();
                    if self.take()? != b':' {
                        return Err("JSON colon".into());
                    }
                    if fields.insert(key, self.value(depth + 1)?).is_some() {
                        return Err("duplicate JSON key".into());
                    }
                    self.space();
                    match self.take()? {
                        b'}' => break,
                        b',' => {}
                        _ => return Err("JSON object separator".into()),
                    }
                }
                Ok(Json::Object(fields))
            }
            Some(b'[') => {
                self.take()?;
                let mut values = Vec::new();
                self.space();
                if self.0.first() == Some(&b']') {
                    self.take()?;
                    return Ok(Json::Array(values));
                }
                loop {
                    values.push(self.value(depth + 1)?);
                    self.space();
                    match self.take()? {
                        b']' => break,
                        b',' => {}
                        _ => return Err("JSON array separator".into()),
                    }
                }
                Ok(Json::Array(values))
            }
            Some(b'n') if self.0.starts_with(b"null") => {
                self.0 = &self.0[4..];
                Ok(Json::Null)
            }
            Some(b'-' | b'0'..=b'9') => {
                let end = self
                    .0
                    .iter()
                    .position(|byte| !matches!(byte, b'-' | b'0'..=b'9'))
                    .unwrap_or(self.0.len());
                let (number, rest) = self.0.split_at(end);
                let text = std::str::from_utf8(number).map_err(|error| error.to_string())?;
                let value: i32 = text
                    .parse()
                    .map_err(|error: std::num::ParseIntError| error.to_string())?;
                if value.to_string() != text {
                    return Err("noncanonical JSON integer".into());
                }
                self.0 = rest;
                Ok(Json::Integer(value))
            }
            _ => Err("unsupported fixture JSON value".into()),
        }
    }
}

#[test]
fn raw_reader_rejects_corruption() {
    assert_eq!(
        parse(r#"{"x":["a\n",0,null]}"#)
            .unwrap()
            .get("x")
            .unwrap()
            .array()
            .unwrap()
            .len(),
        3
    );
    for text in [
        r#"{"x":0,"x":1}"#,
        r#"{"x":01}"#,
        r#"["a\q"]"#,
        r#"[0,]"#,
        r#"{}true"#,
    ] {
        assert!(parse(text).is_err(), "{text}");
    }
    for text in ["0", "GG", "AA"] {
        assert!(hex(text).is_err());
    }
}
