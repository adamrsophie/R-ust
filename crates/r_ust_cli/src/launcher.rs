// shortcut: match macOS R 4.6.0 launcher rules; port platform rules before claiming other platforms.
#[derive(Debug)]
pub(super) enum ExpressionError {
    Invalid,
    TruncatingEscape,
    NonUtf8,
}

impl std::fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Invalid => "-e requires a non-empty expression that does not start with '-'",
            Self::TruncatingEscape => "-e launcher escape \\c is not supported",
            Self::NonUtf8 => "-e launcher escapes produced non-UTF-8 source",
        })
    }
}

impl From<ExpressionError> for String {
    fn from(error: ExpressionError) -> Self {
        error.to_string()
    }
}

pub(super) fn encode_expression(argument: &str) -> Result<String, ExpressionError> {
    if !valid_argument(argument) {
        return Err(ExpressionError::Invalid);
    }
    let mut expanded = match expand_echo(argument) {
        Echo::Complete(bytes) => bytes,
        Echo::Truncated(bytes) if bytes.contains(&b'\n') => bytes,
        Echo::Truncated(_) => return Err(ExpressionError::TruncatingEscape),
    };
    expanded.push(b'\n');
    for _ in 0..2 {
        if expanded.last() == Some(&b'\n') {
            expanded.pop();
        }
    }
    expanded.retain(|byte| *byte != 0);
    let expression = String::from_utf8(expanded).map_err(|_| ExpressionError::NonUtf8)?;
    Ok(expression
        .replace(' ', "~+~")
        .replace('\n', "~n~")
        .replace('\t', "~t~"))
}

enum Echo {
    Complete(Vec<u8>),
    Truncated(Vec<u8>),
}

impl Echo {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Complete(bytes) | Self::Truncated(bytes) => bytes,
        }
    }
}

pub(super) fn fields(argument: &str) -> impl Iterator<Item = &str> {
    argument
        .split([' ', '\t', '\n'])
        .filter(|field| !field.is_empty())
}

pub(super) fn valid_argument(argument: &str) -> bool {
    let unquoted = fields(argument).collect::<Vec<_>>().join(" ");
    expand_echo(&unquoted)
        .into_bytes()
        .split(|byte| *byte == b'\n')
        .any(|line| line.first() != Some(&b'-') && line.iter().any(|byte| *byte != 0))
}

pub(super) fn encode_filename(argument: &str, inline: bool) -> Result<String, ExpressionError> {
    let bytes = expand_echo(argument).into_bytes();
    let text = String::from_utf8(bytes).map_err(|_| ExpressionError::NonUtf8)?;
    let value = if inline {
        text.split_inclusive('\n')
            .map(|line| line.split_once('=').map_or(line, |(_, rest)| rest))
            .collect::<String>()
    } else {
        text
    };
    Ok(value
        .replace('\0', "")
        .trim_end_matches('\n')
        .replace(' ', "~+~"))
}

fn expand_echo(argument: &str) -> Echo {
    let mut bytes = argument.bytes().peekable();
    let mut expanded = Vec::with_capacity(argument.len());
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            expanded.push(byte);
            continue;
        }
        let Some(escape) = bytes.next() else {
            expanded.push(b'\\');
            break;
        };
        let decoded = match escape {
            b'a' => 7,
            b'b' => 8,
            b'f' => 12,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 11,
            b'\\' => b'\\',
            b'c' => return Echo::Truncated(expanded),
            b'0' | b'x' => {
                let radix = if escape == b'0' { 8 } else { 16 };
                let mut value = 0u8;
                let mut digits = 0;
                for _ in 0..if radix == 8 { 3 } else { 2 } {
                    let digit = match bytes.peek() {
                        Some(byte @ b'0'..=b'7') => byte - b'0',
                        Some(byte @ b'8'..=b'9') if radix == 16 => byte - b'0',
                        Some(byte @ b'a'..=b'f') if radix == 16 => byte - b'a' + 10,
                        Some(byte @ b'A'..=b'F') if radix == 16 => byte - b'A' + 10,
                        _ => break,
                    };
                    bytes.next();
                    value = value.wrapping_mul(radix).wrapping_add(digit);
                    digits += 1;
                }
                if escape == b'x' && digits == 0 {
                    expanded.extend_from_slice(b"\\x");
                    continue;
                }
                value
            }
            other => {
                expanded.push(b'\\');
                other
            }
        };
        expanded.push(decoded);
    }
    expanded.push(b'\n');
    Echo::Complete(expanded)
}

pub(super) fn decode_argument(encoded: &str) -> String {
    let mut output = String::with_capacity(encoded.len());
    let mut chars = encoded.chars();
    while let Some(ch) = chars.next() {
        if ch == '~' {
            let mut lookahead = chars.clone();
            let decoded = match (lookahead.next(), lookahead.next()) {
                (Some('+'), Some('~')) => Some(' '),
                (Some('n'), Some('~')) => Some('\n'),
                (Some('t'), Some('~')) => Some('\t'),
                _ => None,
            };
            if let Some(decoded) = decoded {
                output.push(decoded);
                chars = lookahead;
                continue;
            }
        }
        output.push(ch);
    }
    output
}

pub(super) fn append_expression(source: &mut String, startup_stdout: &mut Vec<u8>, encoded: &str) {
    if source.len() + encoded.len() + 2 <= 10000 {
        source.push_str(&decode_argument(encoded));
        source.push('\n');
    } else {
        let mut warning =
            format!("WARNING: '-e {encoded}' omitted as input is too long\n").into_bytes();
        warning.truncate(1024 + 128 - 1);
        warning.push(b'\n');
        startup_stdout.extend(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_argument, encode_expression};

    #[test]
    fn expands_echo_escapes_once_and_encodes_whitespace() {
        for (input, expected) in [
            (
                r"a\ab\bc\fd\ne\rf\tg\vh",
                "a\x07b\x08c\x0cd~n~e\rf~t~g\x0bh",
            ),
            (r"a\0101\x42\0c", "aABc"),
            (r"a\01000", "a@0"),
            (r"a\0400b", "ab"),
            (r"a\x4z\x", "a\x04z\\x"),
            (r"a\eb\101c\ud", r"a\eb\101c\ud"),
            (r"a\\nb", r"a\nb"),
            ("α β\n\t", "α~+~β~n~~t~"),
            ("a\\", "a\\"),
        ] {
            assert_eq!(encode_expression(input).unwrap(), expected, "{input:?}");
        }
        assert_eq!(decode_argument("a~+~b~n~~t~~+~~n~"), "a b\n\t \n");
    }

    #[test]
    fn rejects_empty_negative_truncated_and_non_utf8_expressions() {
        for input in ["", " \n\t", "-1", " -1", r"\00551", r"a\c", r"a\xff"] {
            assert!(encode_expression(input).is_err(), "{input:?}");
        }
    }
}
