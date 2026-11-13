use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[path = "support/run_length.rs"]
mod support;

fn decode(hex: &str) -> String {
    assert_eq!(hex.len() % 2, 0);
    assert!(hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let bytes: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn base_run_length_closures_preserve_forcing_aliases_and_private_kernels() {
    let mut lines = include_str!("fixtures/run-length-boundary-r-4.6.0.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("id\tsource\tstatus\tvalue_or_error\tstdout_hex\twarnings_hex")
    );
    let rows: Vec<_> = lines.collect();
    assert_eq!(rows.len(), 16);
    let mut ids = std::collections::HashSet::new();
    let mut failures = Vec::new();
    for row in rows {
        let fields: [&str; 6] = row
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .expect("six boundary fixture fields");
        assert!(!fields[0].is_empty() && ids.insert(fields[0]));
        assert!(!fields[1].is_empty() && !fields[3].is_empty());
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(1)]))
            .unwrap();
        let result = frontend.eval_source(fields[1]);
        let actual = match result {
            Ok(values) => (
                "OK",
                support::encode(values.last().expect("last expression result")),
            ),
            Err(error) => ("ERR", support::error_payload(error)),
        };
        assert!(matches!(fields[2], "OK" | "ERR"));
        let stdout = decode(fields[4]);
        let mut expected_events = Vec::new();
        if !stdout.is_empty() {
            expected_events.push(OutputEvent::Stdout(stdout));
        }
        expected_events.extend(support::warning_events(&decode(fields[5])));
        let events = frontend.runtime_mut().take_output();
        if actual != (fields[2], fields[3].into()) || events != expected_events {
            failures.push(format!("{}: actual={actual:?}, events={events:?}; expected={:?}, events={expected_events:?}", fields[0], (fields[2],fields[3])));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
