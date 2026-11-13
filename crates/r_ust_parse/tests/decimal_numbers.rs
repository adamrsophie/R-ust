use r_ust_parse::{ParseStatus, Parser};
use r_ust_sexp::{is_na_real, Sexp};

const FIXTURES: &str = include_str!("fixtures/decimal-literals-r-4.6.0.tsv");

fn fingerprint(value: &Sexp) -> String {
    match value {
        Sexp::Real(values) if values.len() == 1 => {
            let value = values[0];
            if is_na_real(value) {
                "double:NA".into()
            } else if value.is_nan() {
                "double:NaN".into()
            } else {
                format!("double:{:016x}", value.to_bits())
            }
        }
        Sexp::Integer(values) if values.len() == 1 => {
            format!(
                "integer:{}",
                values[0].map_or_else(|| "NA".into(), |v| v.to_string())
            )
        }
        other => panic!("expected one numeric literal, got {other:?}"),
    }
}

fn check_fixture(row: &str) {
    let fields: Vec<_> = row.split('\t').collect();
    assert_eq!(fields.len(), 3);
    let source = fields[0];
    let warnings: Vec<_> = fields[2].split('|').filter(|s| *s != "-").collect();
    for result in [
        Parser.parse_all(source),
        Parser.parse_iter(source).next().unwrap(),
    ] {
        match result {
            Ok(parsed) => {
                assert_eq!(parsed.expressions.len(), 1);
                assert_eq!(
                    fingerprint(&parsed.expressions[0]),
                    fields[1],
                    "length {}: {:.80}",
                    source.len(),
                    source
                );
                assert_eq!(parsed.warnings, warnings, "{source}");
            }
            Err(error) => {
                assert_eq!(
                    fields[1],
                    "Error",
                    "length {}: {:.80}: {error:?}",
                    source.len(),
                    source
                );
                assert_eq!(error.status, ParseStatus::Error, "{source}");
                assert_eq!(error.warnings, warnings, "{source}");
            }
        }
    }
}

#[test]
fn decimal_literal_values_types_and_warnings_match_r() {
    for row in FIXTURES.lines().filter(|row| !row.starts_with('#')) {
        let source = row.split('\t').next().unwrap();
        if source.len() < 8189 {
            check_fixture(row);
        }
    }
}

#[test]
fn numeric_token_buffer_limit_includes_hex_prefixes_and_integer_suffixes() {
    for row in FIXTURES.lines().filter(|row| !row.starts_with('#')) {
        let source = row.split('\t').next().unwrap();
        if source.len() >= 8189 {
            check_fixture(row);
        }
    }
}
