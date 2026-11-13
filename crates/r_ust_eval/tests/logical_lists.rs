use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

fn decode_hex(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    assert!(text.len().is_multiple_of(2));
    String::from_utf8(
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect(),
    )
    .unwrap()
}

#[test]
fn logical_lists_match_independent_r_4_6_0_fixture() {
    let fixture = include_str!("fixtures/logical-lists-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 JIT 0")
    );
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        let mut frontend = Frontend::default();
        let (kind, payload, error) = match frontend.eval_source(fields[1]) {
            Ok(values) => {
                let Some(Sexp::Logical(values)) = values.last() else {
                    panic!("{}: unexpected storage {values:?}", fields[1]);
                };
                let payload = values
                    .iter()
                    .map(|value| match value {
                        Some(true) => "T",
                        Some(false) => "F",
                        None => "NA",
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                (
                    "logical",
                    if payload.is_empty() {
                        "-".into()
                    } else {
                        payload
                    },
                    String::new(),
                )
            }
            Err(EvalError::Type(message)) => ("error", "-".into(), message),
            result => panic!("{}: unexpected result {result:?}", fields[1]),
        };
        assert_eq!(
            (kind, payload.as_str(), error.as_str()),
            (fields[2], fields[3], decode_hex(fields[4]).as_str()),
            "{}",
            fields[1]
        );
        assert_eq!(fields[5], "-");
        assert!(
            frontend.runtime_mut().take_output().is_empty(),
            "{}",
            fields[1]
        );
        println!(
            "LOGICAL_ROW\t{}\tPASS\t{}\t{kind}\t{payload}",
            fields[0], fields[1]
        );
    }
    assert_eq!(fixture.lines().count(), 25);
}
