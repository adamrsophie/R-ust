use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::Sexp;

#[test]
fn raw_literals_preserve_values_and_incremental_offsets() {
    for prefix in ['r', 'R'] {
        for quote in ['\'', '"'] {
            for (open, close) in [('(', ')'), ('[', ']'), ('{', '}'), ('|', '|')] {
                for dashes in ["", "-", "---"] {
                    for value in [
                        "",
                        r"C:\path\n\z\u0041",
                        "both ' and \" quotes",
                        "α😀\n# ; ()[]{}||",
                        "a\r\nb",
                    ] {
                        let source =
                            format!("{prefix}{quote}{dashes}{open}{value}{close}{dashes}{quote}");
                        let expected = vec![Sexp::String(vec![Some(value.to_string())])];
                        assert_eq!(
                            Parser.parse_all(&source).unwrap().expressions,
                            expected,
                            "{source:?}"
                        );
                        let mut statements = Parser.parse_iter(&source);
                        assert_eq!(
                            statements.next().unwrap().unwrap().expressions,
                            expected,
                            "{source:?}"
                        );
                        assert_eq!(statements.consumed_bytes(), source.len());
                    }
                }
            }
        }
    }
}

#[test]
fn terminator_requires_exact_dashes_and_quote() {
    for quote in ['\'', '"'] {
        let other = if quote == '\'' { '"' } else { '\'' };
        for (open, close) in [('(', ')'), ('[', ']'), ('{', '}'), ('|', '|')] {
            let value = format!("{close}-{quote}X{close}---{quote}Y{close}--{other}");
            let source = format!("r{quote}--{open}{value}{close}--{quote}");
            assert_eq!(
                Parser.parse_all(&source).unwrap().expressions,
                vec![Sexp::String(vec![Some(value)])],
                "{source:?}"
            );
        }
    }
}

#[test]
fn malformed_openers_and_incomplete_bodies_have_distinct_statuses() {
    for source in [
        "r\"\"",
        "r\"x\"",
        "r\"-x\"",
        "R\"",
        "r\"--\"",
        "r\"<text>\"",
        "r'wrong'",
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Error,
            "{source:?}"
        );
    }
    for source in [
        "r\"(",
        "r\"-[text",
        "R'--{text}--\"",
        "r\"|text",
        "r\"-(text)\"",
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Incomplete,
            "{source:?}"
        );
    }
    assert!(Parser.parse_all("r<-1;R<-2;rate<-r+R").is_ok());
}

#[test]
fn raw_values_reject_literal_bidi_and_nul_but_allow_escape_text() {
    for character in ['\0', '\u{202a}', '\u{202e}', '\u{2066}', '\u{2069}'] {
        let source = format!("r\"(a{character}b)\"");
        assert_eq!(
            Parser.parse_all(&source).unwrap_err().status,
            ParseStatus::Error
        );
    }
    let value = r"\0\u202e\xFF\c";
    assert_eq!(
        Parser
            .parse_all(&format!("r\"({value})\""))
            .unwrap()
            .expressions,
        vec![Sexp::String(vec![Some(value.to_string())])]
    );
}
