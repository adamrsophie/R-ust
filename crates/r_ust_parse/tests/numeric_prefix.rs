use r_ust_parse::{parse_numeric_prefix, Parser};
use r_ust_sexp::Sexp;

#[test]
fn numeric_prefix_values_and_suffixes_match_r() {
    for row in include_str!("fixtures/numeric-prefix-r-4.6.0.tsv")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 2);
        for suffix in ["", "λ trailing"] {
            let input = format!("{}{suffix}", fields[0]);
            let (value, rest) = parse_numeric_prefix(&input).unwrap();
            if fields[1] == "NaN" {
                assert!(value.is_nan());
            } else {
                let bits = u64::from_str_radix(fields[1], 16).unwrap();
                assert_eq!(value.to_bits(), bits, "{:.80}", input);
            }
            assert_eq!(rest, suffix);
            assert!(std::ptr::eq(
                rest.as_ptr(),
                input[fields[0].len()..].as_ptr()
            ));
        }
    }
}

#[test]
fn string_grammar_is_separate_from_literal_grammar() {
    for (input, expected, suffix) in [
        (" \t\n\r\u{b}\u{c}-0tail", -0.0, "tail"),
        ("0x", 0.0, "x"),
        ("-1e-9999", -0.0, ""),
        ("0x ", 0.0, " "),
        ("0xp0", 0.0, ""),
        ("0x.p0", 0.0, ""),
        ("0x1.2.3", 18.1875, ""),
        ("0x1..2", 1.125, ""),
        ("1.25.6", 1.25, ".6"),
        ("+InFiNiTy!", f64::INFINITY, "!"),
        ("-INFé", f64::NEG_INFINITY, "é"),
    ] {
        let (value, rest) = parse_numeric_prefix(input).unwrap();
        assert_eq!(value.to_bits(), expected.to_bits(), "{input:?}");
        assert_eq!(rest, suffix);
        assert!(std::ptr::eq(
            rest.as_ptr(),
            input[input.len() - suffix.len()..].as_ptr()
        ));
    }
    for input in [
        "",
        " ",
        ".",
        "NA",
        "+",
        "- 1",
        "--1",
        "1e",
        "1e+λ",
        "1e-",
        "0xp",
        "0x1p+",
        "\u{2000}1",
        "\u{85}1",
    ] {
        assert!(parse_numeric_prefix(input).is_none(), "{input:?}");
    }
    for input in ["0xp0", "0x1.2.3", "0x1..2"] {
        assert!(Parser.parse_all(input).is_err(), "{input}");
        assert!(Parser.parse_iter(input).next().unwrap().is_err(), "{input}");
    }
    let input = format!("0.{}1e100000", "0".repeat(9999));
    let (value, suffix) = parse_numeric_prefix(&input).unwrap();
    assert_eq!(value.to_bits(), 1.0_f64.to_bits());
    assert_eq!(suffix, "");
}

#[test]
fn numeric_spans_follow_unicode_and_raw_string_checkpoints() {
    for prefix in ["\"α😀\"", "`α😀`", "r\"--(α)-x😀)--\""] {
        let source = format!("{prefix};1.25;0x1.8p1");
        let parsed = Parser.parse_all(&source).unwrap();
        assert_eq!(
            parsed.expressions[1..],
            [Sexp::Real(vec![1.25]), Sexp::Real(vec![3.0])]
        );
        let mut statements = Parser.parse_iter(&source);
        statements.next().unwrap().unwrap();
        for (value, consumed) in [(1.25, prefix.len() + 6), (3.0, source.len())] {
            assert_eq!(
                statements.next().unwrap().unwrap().expressions,
                vec![Sexp::Real(vec![value])]
            );
            assert_eq!(statements.consumed_bytes(), consumed);
        }
        assert!(statements.next().is_none());
    }
}
