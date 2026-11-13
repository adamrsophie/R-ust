use r_ust_sexp::{is_na_real, Sexp, NA_REAL};

use super::atomic::{integers, reals, scalar_integer};
use super::strings::BlankPolicy;
use super::warnings::Warnings;

fn input(encoded: &str) -> Option<String> {
    match encoded {
        "NA" => None,
        "-" => Some(String::new()),
        encoded => {
            assert_eq!(encoded.len() % 2, 0);
            let bytes = (0..encoded.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16).unwrap())
                .collect();
            Some(String::from_utf8(bytes).unwrap())
        }
    }
}

fn flags(warnings: Warnings) -> String {
    let mut result = String::new();
    if warnings.generic {
        result.push('G');
    }
    if warnings.range {
        result.push('R');
    }
    if result.is_empty() {
        result.push('-');
    }
    result
}

#[test]
fn numeric_string_values_and_warning_categories_match_independent_r() {
    for (policy, fixture) in [
        (
            BlankPolicy::C,
            include_str!("../../tests/fixtures/numeric-coercion-C-r-4.6.0.tsv"),
        ),
        (
            BlankPolicy::Utf8,
            include_str!("../../tests/fixtures/numeric-coercion-Utf8-r-4.6.0.tsv"),
        ),
    ] {
        let mut count = 0;
        for row in fixture.lines().filter(|row| !row.starts_with('#')) {
            let fields: Vec<_> = row.split('\t').collect();
            assert_eq!(fields.len(), 6);
            count += 1;
            assert_eq!(fields[0].parse::<usize>().unwrap(), count);
            let value = Sexp::String(vec![input(fields[1])]);
            let (numbers, warnings) = reals(&value, policy).unwrap();
            assert_eq!(numbers.len(), 1);
            match fields[2] {
                "NA" => assert!(is_na_real(numbers[0]), "case {} {policy:?}", fields[0]),
                "NaN" => assert!(numbers[0].is_nan() && !is_na_real(numbers[0])),
                bits => assert_eq!(
                    numbers[0].to_bits(),
                    u64::from_str_radix(bits, 16).unwrap(),
                    "case {} {policy:?}",
                    fields[0]
                ),
            }
            assert_eq!(flags(warnings), fields[3], "case {} {policy:?}", fields[0]);
            let (numbers, warnings) = integers(&value, policy).unwrap();
            let expected = match fields[4] {
                "NA" => None,
                number => Some(number.parse::<i32>().unwrap()),
            };
            assert_eq!(numbers, [expected], "case {} {policy:?}", fields[0]);
            assert_eq!(flags(warnings), fields[5], "case {} {policy:?}", fields[0]);
        }
        assert_eq!(count, 2061);
    }
}

#[test]
fn scalar_conversion_ignores_tail_and_preserves_numeric_nan() {
    for (value, expected) in [
        (
            Sexp::String(vec![Some("7".into()), Some("bad".into())]),
            Some(7),
        ),
        (Sexp::Real(vec![7.9, f64::INFINITY]), Some(7)),
        (Sexp::Real(vec![f64::NAN, 7.0]), None),
        (Sexp::Real(vec![NA_REAL, 7.0]), None),
        (Sexp::String(vec![Some("NaN".into())]), Some(0)),
    ] {
        let (actual, warnings) = scalar_integer(&value, BlankPolicy::C);
        assert_eq!(actual, expected);
        assert_eq!(warnings, Warnings::default());
    }
}

#[test]
fn locale_selection_honors_first_nonempty_value() {
    for (locales, utf8) in [
        (vec!["C", "en_CA.UTF-8"], false),
        (vec!["", "C.UTF-8", "C"], true),
        (vec!["", "", "en_CA.utf8"], true),
        (vec!["POSIX", "C.UTF-8"], false),
        (vec!["en_CA.ISO8859-1"], false),
    ] {
        let policy = BlankPolicy::locales(locales.into_iter().map(String::from));
        assert_eq!(matches!(policy, BlankPolicy::Utf8), utf8);
    }
}
