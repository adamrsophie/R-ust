// shortcut: match macOS R 4.6.0 echo; add shell-specific rules before claiming other platforms.
pub(super) fn encode_expression(argument: &str) -> Result<String, String> {
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
            b'c' => return Err("-e launcher escape \\c is not supported".to_string()),
            b'0' | b'x' => {
                let radix = if escape == b'0' { 8 } else { 16 };
                let mut value = 0u8;
                let mut digits = 0;
                for _ in 0..if radix == 8 { 3 } else { 2 } {
                    let Some(digit) = bytes
                        .peek()
                        .and_then(|byte| (*byte as char).to_digit(radix))
                    else {
                        break;
                    };
                    bytes.next();
                    value = value.wrapping_mul(radix as u8).wrapping_add(digit as u8);
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
    // Shell command substitution discards NUL bytes before encoding the expression.
    expanded.retain(|byte| *byte != 0);
    let expression = String::from_utf8(expanded)
        .map_err(|_| "-e launcher escapes produced non-UTF-8 source".to_string())?;
    let first_token = expression.trim_start_matches([' ', '\t', '\n']);
    if first_token.is_empty() || first_token.starts_with('-') {
        return Err("-e requires a non-empty expression that does not start with '-'".to_string());
    }
    Ok(expression
        .replace(' ', "~+~")
        .replace('\n', "~n~")
        .replace('\t', "~t~"))
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
