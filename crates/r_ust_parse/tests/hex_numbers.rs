use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::Sexp;

#[test]
fn hexadecimal_values_match_r_conversion_in_both_parser_apis() {
    for (source, value) in [
        ("0x1F", 31.0_f64),
        ("0Xff", 255.0),
        ("0x1e3", 483.0),
        ("0x1.8", 1.5),
        ("0x.8", 0.5),
        ("0x.", 0.0),
        ("0x.p1", 0.0),
        ("0x10.4", 16.25),
        ("0x1.p2", 4.0),
        ("0X1P-1", 0.5),
        ("0x1p-1023", f64::from_bits(0x0008_0000_0000_0000)),
        ("0x1p-1024", 0.0),
        ("0x1p-1074", 0.0),
        ("0x1p1024", f64::INFINITY),
        ("0x1.fffffffffffffp1023", f64::MAX),
        ("0x1000000000000081", 2.0_f64.powi(60)),
        ("0x0p999999999999999999999999", 0.0),
        ("0x1p999999999999999999999999", f64::INFINITY),
        ("0x1p-999999999999999999999999", 0.0),
    ] {
        let expected = vec![Sexp::Real(vec![value])];
        assert_eq!(
            Parser.parse_all(source).unwrap().expressions,
            expected,
            "{source}"
        );
        let mut statements = Parser.parse_iter(source);
        assert_eq!(
            statements.next().unwrap().unwrap().expressions,
            expected,
            "{source}"
        );
        assert_eq!(statements.consumed_bytes(), source.len());
    }
}

#[test]
fn hexadecimal_integer_qualifiers_reuse_numeric_type_rules() {
    for (source, expected) in [
        ("0x1L", Sexp::Integer(vec![Some(1)])),
        ("0x7fffffffL", Sexp::Integer(vec![Some(i32::MAX)])),
        ("0x1p2L", Sexp::Integer(vec![Some(4)])),
        ("0x1.0L", Sexp::Integer(vec![Some(1)])),
        ("0x80000000L", Sexp::Real(vec![2147483648.0])),
        ("0x1.8L", Sexp::Real(vec![1.5])),
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap().expressions,
            vec![expected],
            "{source}"
        );
    }
}

#[test]
fn malformed_hexadecimal_numbers_are_errors() {
    for source in [
        "0x", "0X", "0xp1", "0xL", "0xg", "0x1p", "0x1p+", "0x1p-", "0x.p", "0x1.2.3", "0x1p--1",
        "0x1p1.0", "0x1p1e2", "0x1p1p2",
    ] {
        assert_eq!(
            Parser.parse_all(source).unwrap_err().status,
            ParseStatus::Error,
            "{source}"
        );
        assert_eq!(
            Parser
                .parse_iter(source)
                .next()
                .unwrap()
                .unwrap_err()
                .status,
            ParseStatus::Error,
            "{source}"
        );
    }
}
