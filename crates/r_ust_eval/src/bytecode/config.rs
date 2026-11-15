pub(crate) fn enabled() -> bool {
    std::env::var("R_DISABLE_BYTECODE")
        .map(|value| !positive_prefix(&value))
        .unwrap_or(true)
}

fn positive_prefix(value: &str) -> bool {
    let value = value.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    let value = value.strip_prefix('+').unwrap_or(value);
    value
        .bytes()
        .take_while(u8::is_ascii_digit)
        .any(|digit| digit != b'0')
}

#[cfg(test)]
mod tests {
    use super::positive_prefix;

    #[test]
    fn disable_flag_uses_c_decimal_prefix_and_sign() {
        for value in ["1", "  +2tail", "0001x", "\t3", "01"] {
            assert!(positive_prefix(value), "{value}");
        }
        for value in ["", "0", "-1", "  -2", "0x1", "false", "+x", "\u{a0}1"] {
            assert!(!positive_prefix(value), "{value}");
        }
    }
}
