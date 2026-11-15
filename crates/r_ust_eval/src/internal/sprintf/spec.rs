#[derive(Clone, Debug)]
pub(super) struct Conversion {
    pub text: String,
    pub argument: usize,
    pub star: Option<usize>,
}

impl Conversion {
    pub fn parse(raw: &str, cursor: &mut usize, count: usize) -> Result<Self, String> {
        let mut text = raw.to_owned();
        let argument = position(&mut text, 1)?;
        if let Some(position) = argument.filter(|position| *position > count) {
            return Err(format!("reference to non-existent argument {position}"));
        }
        let star = if let Some(offset) = text.find('*') {
            let specified = position(&mut text, offset + 1)?;
            let index = select(specified, cursor, count)?;
            if text[offset + 1..].contains('*') {
                return Err(
                    "at most one asterisk '*' is supported in each conversion specification".into(),
                );
            }
            Some(index)
        } else {
            None
        };
        let argument = select(argument, cursor, count)?;
        Ok(Self {
            text,
            argument,
            star,
        })
    }
}

fn position(text: &mut String, offset: usize) -> Result<Option<usize>, String> {
    let bytes = text.as_bytes();
    if !bytes
        .get(offset)
        .is_some_and(|byte| (b'1'..=b'9').contains(byte))
    {
        return Ok(None);
    }
    for digits in [1, 2] {
        if bytes.get(offset + digits) == Some(&b'$') {
            let number = text[offset..offset + digits]
                .parse::<usize>()
                .map_err(|_| "invalid argument position".to_owned())?;
            text.replace_range(offset..offset + digits + 1, "");
            return Ok(Some(number));
        }
        if !bytes.get(offset + digits).is_some_and(u8::is_ascii_digit) {
            break;
        }
    }
    Ok(None)
}

fn select(position: Option<usize>, cursor: &mut usize, count: usize) -> Result<usize, String> {
    if let Some(position) = position {
        if position > count {
            return Err(format!("reference to non-existent argument {position}"));
        }
        return Ok(position - 1);
    }
    if *cursor >= count {
        return Err("too few arguments".into());
    }
    let index = *cursor;
    *cursor += 1;
    Ok(index)
}

#[derive(Clone, Debug, Default)]
pub(super) struct Spec {
    pub text: String,
    pub conversion: char,
    pub valid: bool,
    pub left: bool,
    pub plus: bool,
    pub space: bool,
    pub alternate: bool,
    pub zero: bool,
    pub width: usize,
    pub precision: Option<usize>,
}

impl Spec {
    pub fn parse(text: String) -> Result<Self, String> {
        let mut spec = Self {
            text: text.clone(),
            valid: true,
            ..Self::default()
        };
        let bytes = text.as_bytes();
        let mut cursor = 1;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'-' => spec.left = true,
                b'+' => spec.plus = true,
                b' ' => spec.space = true,
                b'#' => spec.alternate = true,
                b'0' => spec.zero = true,
                b'.' => {
                    cursor += 1;
                    spec.precision = Some(number(bytes, &mut cursor)?);
                    continue;
                }
                b'1'..=b'9' => {
                    spec.width = number(bytes, &mut cursor)?;
                    continue;
                }
                byte => {
                    spec.conversion = char::from(byte);
                    spec.valid = cursor + 1 == bytes.len();
                    break;
                }
            }
            cursor += 1;
        }
        Ok(spec)
    }

    pub fn sign(&self, negative: bool) -> &'static str {
        if negative {
            "-"
        } else if self.plus {
            "+"
        } else if self.space {
            " "
        } else {
            ""
        }
    }

    pub fn error(&self, hint: &str) -> String {
        format!("invalid format '{}'; {hint}", self.text)
    }
}

fn number(bytes: &[u8], cursor: &mut usize) -> Result<usize, String> {
    let mut result = 0usize;
    while let Some(byte @ b'0'..=b'9') = bytes.get(*cursor) {
        result = result
            .checked_mul(10)
            .and_then(|n| n.checked_add(usize::from(*byte - b'0')))
            .ok_or_else(|| "format width or precision is too large".to_owned())?;
        *cursor += 1;
    }
    Ok(result)
}
