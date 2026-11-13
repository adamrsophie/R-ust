use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;
use std::fmt::Write as _;

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

fn decode(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            assert!(pair.iter().all(u8::is_ascii_hexdigit));
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
        })
        .collect()
}

fn fingerprint(value: &Sexp) -> (String, String) {
    let (kind, len, data) = match value {
        Sexp::Nil => return ("NULL".into(), "NULL".into()),
        Sexp::Logical(values) => (
            "logical",
            values.len(),
            values
                .iter()
                .map(|value| match value {
                    Some(true) => "T",
                    Some(false) => "F",
                    None => "NA",
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::Integer(values) => (
            "integer",
            values.len(),
            values
                .iter()
                .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::String(values) => (
            "character",
            values.len(),
            values
                .iter()
                .map(|value| {
                    value.as_ref().map_or_else(
                        || "NA".into(),
                        |value| format!("x{}", hex(value.as_bytes())),
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        other => panic!("unexpected logical alias fixture value {other:?}"),
    };
    (kind.into(), format!("{kind}|{len}|{data}"))
}

#[test]
fn logical_aliases_match_independent_r_4_6_0_bindings() {
    let fixture = include_str!("fixtures/logical-aliases-r-4.6.0.tsv");
    assert_eq!(fixture.lines().next(), Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"));
    let mut counts = [0; 5];
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7, "{line}");
        let id: usize = fields[0].parse().unwrap();
        assert_eq!(id, index + 1);
        let slot = match fields[2] {
            "logical" => 0,
            "integer" => 1,
            "character" => 2,
            "NULL" => 3,
            "error" => 4,
            other => panic!("unknown oracle type {other}"),
        };
        counts[slot] += 1;
        let expected_error = decode(fields[4]);
        assert_eq!(fields[2] == "error", !expected_error.is_empty());
        assert_eq!(fields[2] == "error", fields[3] == "-");
        assert_eq!(fields[5], "-", "unexpected reference warning");
        assert_eq!(fields[6], "-", "unexpected reference output");
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[1]);
        let (kind, value, error) = match &result {
            Ok(values) => {
                let (kind, value) = fingerprint(values.last().unwrap());
                (kind, value, Vec::new())
            }
            Err(error) => ("error".into(), "-".into(), error.to_string().into_bytes()),
        };
        let output = frontend.runtime_mut().take_output();
        let passed =
            kind == fields[2] && value == fields[3] && error == expected_error && output.is_empty();
        println!(
            "ROW\t{id}\t{}\t{}\t{kind}\t{value}\t{}",
            if passed { "PASS" } else { "FAIL" },
            fields[1],
            hex(&error)
        );
        if !passed {
            failures.push(format!(
                "row {id}: {result:?}, output {output:?}; expected {line}"
            ));
        }
    }
    assert_eq!(counts, [12, 32, 4, 2, 4]);
    assert_eq!(fixture.lines().count(), 55);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
