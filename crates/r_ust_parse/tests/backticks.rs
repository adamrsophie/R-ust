use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::Sexp;

#[test]
fn backticks_decode_names_without_recognizing_keywords() {
    for (body, name) in [
        ("my var", "my var"),
        ("if", "if"),
        ("function", "function"),
        ("TRUE", "TRUE"),
        ("NULL", "NULL"),
        ("NA_real_", "NA_real_"),
        ("α😀", "α😀"),
        (r"a\`b\\c", "a`b\\c"),
        (r"\a\b\f\n\r\t\v", "\x07\x08\x0c\n\r\t\x0b"),
        (r#"\'\"\ "#, "'\" "),
        (r"\1\12\1234\x41b", "\x01\nS4Ab"),
        (r"\303\251", "é"),
        (r"\xC3\xA9", "é"),
        ("a\r\nb", "a\r\nb"),
        ("a\\\nb", "a\nb"),
    ] {
        let source = format!("`{body}`");
        let expected = vec![Sexp::Symbol(name.to_string())];
        assert_eq!(Parser.parse_all(&source).unwrap().expressions, expected);
        let mut statements = Parser.parse_iter(&source);
        assert_eq!(statements.next().unwrap().unwrap().expressions, expected);
        assert_eq!(statements.consumed_bytes(), source.len());
    }
}

#[test]
fn quoted_names_work_in_each_name_position() {
    for source in [
        "`my var`<-3;`my var`+1",
        "f<-function(`if`, `a b`=2) `if`+`a b`; f(`if`=1,`a b`=4)",
        "f(`a b`=)",
        "for (`loop var` in 1:3) `loop var`",
        "x$`a b`<-1;x$`a b`",
        "`f name`<-function(x)x;`f name`(1)",
        "`x`<<-1;2->>`x`",
    ] {
        assert!(Parser.parse_all(source).is_ok(), "{source}");
        assert!(
            Parser.parse_iter(source).all(|result| result.is_ok()),
            "{source}"
        );
    }
    for source in ["if(TRUE) 1 `else` 2", "for(x `in` 1:3) x"] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Error
        );
    }
}

#[test]
fn invalid_backticks_are_errors_and_unclosed_names_are_incomplete() {
    for body in [
        "",
        r"\u0061",
        r"\u{61}",
        r"\U00000061",
        r"\u",
        r"\U",
        r"\z",
        r"\0",
        r"\x00",
        r"\400",
        "\0",
        "\u{202e}",
        "\u{2066}",
    ] {
        let source = format!("`{body}`");
        assert_eq!(
            Parser.parse_all(&source).unwrap_err().status,
            ParseStatus::Error,
            "{source:?}"
        );
    }
    for source in [
        "`",
        "`unfinished",
        "`trailing\\",
        "`\\x41",
        "`a\n",
        "`\\x",
        "`\\x0",
        "`\\0",
        "`\\00",
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Incomplete,
            "{source:?}"
        );
    }
    for source in ["`\\u", "`\\U"] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Error,
            "{source:?}"
        );
    }
}

#[test]
fn name_limit_counts_decoded_bytes_not_source_or_characters() {
    for name in ["x".repeat(10000), "é".repeat(5000)] {
        assert_eq!(
            Parser.parse_all(&format!("`{name}`")).unwrap().expressions,
            vec![Sexp::Symbol(name.clone())]
        );
        assert_eq!(
            Parser.parse_all(&format!("`{name}x`")).unwrap_err().status,
            ParseStatus::Error
        );
    }
    let body = r"\x61".repeat(10000);
    assert_eq!(
        Parser.parse_all(&format!("`{body}`")).unwrap().expressions,
        vec![Sexp::Symbol("a".repeat(10000))]
    );
}
