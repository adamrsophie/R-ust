use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::Sexp;

#[test]
fn quoted_strings_decode_r_escapes_in_both_parser_apis() {
    for quote in ['\'', '"'] {
        for (body, expected) in [
            (r"\a\b\f\n\r\t\v", "\x07\x08\x0c\n\r\t\x0b"),
            (r#"\\\'\"\`\ "#, "\\'\"` "),
            ("a\\\nb", "a\nb"),
            (r"\1\12\1234\x41b\x4z", "\x01\nS4Ab\x04z"),
            (r"\303\251", "é"),
            (r"\xC3\xA9", "é"),
            (r"\u1\u00e9\u{3b1}", "\x01éα"),
            (r"\U1\U0001f600\U{10ffff}", "\x01😀\u{10ffff}"),
            (r"\uD83D\uDE00", "😀"),
            (r"é\u0061\U{1f600}", "éa😀"),
            (r"\u202e", "\u{202e}"),
        ] {
            let source = format!("{quote}{body}{quote}");
            let expected = vec![Sexp::String(vec![Some(expected.to_string())])];
            assert_eq!(
                Parser.parse_all(&source).unwrap().expressions,
                expected,
                "{source}"
            );
            let actual = Parser.parse_iter(&source).next().unwrap().unwrap();
            assert_eq!(actual.expressions, expected, "{source}");
        }
    }
}

#[test]
fn invalid_escapes_are_errors_and_unclosed_strings_remain_incomplete() {
    for body in [
        r"\z",
        r"\e",
        r"\8",
        r"\x",
        r"\u",
        r"\U",
        r"\000",
        r"\x00",
        r"\u0000",
        r"\U00000000",
        r"\400",
        r"\777",
        r"\U00110000",
        r"\u{00001}",
        r"\U{000000001}",
        r"\u{a",
        r"\u{}",
        r"\u0061\x41",
        r"\101\u0061",
        "\u{202e}",
    ] {
        let source = format!("\"{body}\"");
        assert_eq!(
            Parser.parse_all(&source).unwrap_err().status,
            ParseStatus::Error,
            "{source}"
        );
    }
    for source in ["\"unfinished", "\"trailing\\", r#""\u{a}"#, r#""\U123"#] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Incomplete,
            "{source}"
        );
    }
}
