mod tables;

pub(super) fn encode(value: &str, quote: Option<char>) -> String {
    let mut result = String::new();
    result.extend(quote);
    for character in value.chars() {
        match character {
            '\\' => result.push_str("\\\\"),
            '\u{7}' => result.push_str("\\a"),
            '\u{8}' => result.push_str("\\b"),
            '\u{c}' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{b}' => result.push_str("\\v"),
            '\0' => result.push_str("\\0"),
            '\'' | '"' | '`' if Some(character) == quote => {
                result.push('\\');
                result.push(character);
            }
            character if character.is_ascii_control() => {
                result.push_str(&format!("\\{:03o}", u32::from(character)))
            }
            character
                if !tables::PRINTABLE
                    .iter()
                    .any(|&(start, end)| (start..=end).contains(&u32::from(character))) =>
            {
                let code = u32::from(character);
                result.push_str(&if code > 0xffff {
                    format!("\\U{{{code:06x}}}")
                } else {
                    format!("\\u{code:04x}")
                });
            }
            character => result.push(character),
        }
    }
    result.extend(quote);
    result
}

pub(super) fn width(value: &str) -> usize {
    value
        .chars()
        .map(|character| {
            let code = u32::from(character);
            if let Some(&(_, _, width)) = tables::WIDTHS
                .iter()
                .find(|&&(start, end, _)| (start..=end).contains(&code))
            {
                return width;
            }
            if tables::ZERO
                .iter()
                .any(|&(start, end)| (start..=end).contains(&code))
            {
                0
            } else {
                1
            }
        })
        .sum()
}
