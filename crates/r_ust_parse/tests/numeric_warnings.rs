use r_ust_parse::{ParseStatus, Parser};

#[test]
fn numeric_qualifier_warnings_keep_original_literal_spelling() {
    for (source, warning) in [
        (
            "1.5L",
            "integer literal 1.5L contains decimal; using numeric value",
        ),
        (
            "1.0L",
            "integer literal 1.0L contains unnecessary decimal point",
        ),
        (
            "1e-1L",
            "non-integer value 1e-1L qualified with L; using numeric value",
        ),
        (
            "1.5e0L",
            "non-integer value 1.5e0L qualified with L; using numeric value",
        ),
        (
            "0x1.8L",
            "integer literal 0x1.8L contains decimal; using numeric value",
        ),
        (
            "0X01.0L",
            "integer literal 0X01.0L contains unnecessary decimal point",
        ),
        (
            "0x1p-1L",
            "non-integer value 0x1p-1L qualified with L; using numeric value",
        ),
        (
            "2147483648L",
            "non-integer value 2147483648L qualified with L; using numeric value",
        ),
    ] {
        let expected = vec![warning.to_string()];
        assert_eq!(Parser.parse_all(source).unwrap().warnings, expected);
        assert_eq!(
            Parser.parse_iter(source).next().unwrap().unwrap().warnings,
            expected
        );
    }
    for source in [
        "1L",
        "1e0L",
        "1.0e0L",
        "0x1eL",
        "0x1.0p0L",
        "0x1p-1024L",
        "\"1.5L\"",
        "r\"(1.5L)\"",
        "`1.5L`",
        "# 1.5L\n",
    ] {
        assert!(
            Parser.parse_all(source).unwrap().warnings.is_empty(),
            "{source}"
        );
    }
}

#[test]
fn incremental_warnings_belong_to_each_complete_statement() {
    let mut statements = Parser.parse_iter("1.5L;1.0L");
    assert_eq!(
        statements.next().unwrap().unwrap().warnings,
        vec!["integer literal 1.5L contains decimal; using numeric value"]
    );
    assert_eq!(
        statements.next().unwrap().unwrap().warnings,
        vec!["integer literal 1.0L contains unnecessary decimal point"]
    );
    assert!(statements.next().is_none());
    let parsed = Parser.parse_iter("1.5L+\n2.5L").next().unwrap().unwrap();
    assert_eq!(parsed.warnings.len(), 2);
    assert_eq!(
        Parser.parse_all("1.5L+\n2.5L").unwrap().warnings,
        parsed.warnings
    );
}

#[test]
fn failed_and_incomplete_parses_preserve_earlier_warning_metadata() {
    for (source, status) in [
        ("1.5L+@", ParseStatus::Error),
        ("1.5L+", ParseStatus::Incomplete),
    ] {
        let expected =
            vec!["integer literal 1.5L contains decimal; using numeric value".to_string()];
        let error = Parser.parse_all(source).unwrap_err();
        assert_eq!(error.status, status);
        assert_eq!(error.warnings, expected);
        let error = Parser.parse_iter(source).next().unwrap().unwrap_err();
        assert_eq!(error.status, status);
        assert_eq!(error.warnings, expected);
    }
}
