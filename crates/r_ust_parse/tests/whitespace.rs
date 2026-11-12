use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::Sexp;

#[test]
fn r_blank_characters_work_in_eager_and_incremental_parsing() {
    for blank in [
        ' ', '\t', '\x0c', '\u{1680}', '\u{2000}', '\u{2006}', '\u{2008}', '\u{200a}', '\u{205f}',
        '\u{3000}',
    ] {
        assert_eq!(
            Parser.parse_all(&blank.to_string()).unwrap().status,
            ParseStatus::Null
        );
        let source = format!("1{blank}+2");
        let expected = Parser.parse_all(&source).unwrap();
        assert_eq!(expected.expressions.len(), 1);
        let mut statements = Parser.parse_iter(&source);
        assert_eq!(statements.next().unwrap().unwrap(), expected);
        assert_eq!(statements.consumed_bytes(), source.len());
    }
}

#[test]
fn nonblank_whitespace_is_rejected_outside_strings() {
    for character in [
        '\r', '\x0b', '\u{85}', '\u{a0}', '\u{2007}', '\u{2028}', '\u{2029}', '\u{202f}',
    ] {
        for source in [character.to_string(), format!("1{character}+2")] {
            assert_eq!(
                Parser.parse_all(&source).unwrap_err().status,
                ParseStatus::Error,
                "{source:?}"
            );
            assert_eq!(
                Parser
                    .parse_iter(&source)
                    .next()
                    .unwrap()
                    .unwrap_err()
                    .status,
                ParseStatus::Error,
                "{source:?}"
            );
        }
        let value = format!("a{character}b");
        assert_eq!(
            Parser
                .parse_all(&format!("\"{value}\""))
                .unwrap()
                .expressions,
            vec![Sexp::String(vec![Some(value)])]
        );
    }
}

#[test]
fn crlf_keeps_original_incremental_byte_offsets() {
    let source = "\r\n1\r\n2\r\n";
    let expected = vec![Sexp::Real(vec![1.0]), Sexp::Real(vec![2.0])];
    assert_eq!(Parser.parse_all(source).unwrap().expressions, expected);
    let mut statements = Parser.parse_iter(source);
    let actual = statements
        .by_ref()
        .flat_map(|parsed| parsed.unwrap().expressions)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(statements.consumed_bytes(), source.len());
}
