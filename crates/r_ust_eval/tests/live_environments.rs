use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn boolean_token(token: &str, field: &str) -> bool {
    match token {
        "TRUE" => true,
        "FALSE" => false,
        other => panic!("invalid {field} fixture token: {other}"),
    }
}

#[test]
fn direct_top_level_r_oracle_matches_live_lexical_behavior() {
    let rows = include_str!("fixtures/live-environments-r-4.6.0.tsv");
    let mut lines = rows.lines();
    assert_eq!(
        lines.next(),
        Some("# R 4.6.0, JIT disabled, withVisible block evaluated directly at top level."),
        "fixture header"
    );
    let names = [
        "sibling",
        "counter",
        "late_global",
        "late_local",
        "local_force",
        "late_default",
        "default_local",
        "supplied_global",
        "forwarded",
        "paired",
        "independent",
        "lexical",
        "nearest",
        "reentrant",
        "recursive",
        "function_mode",
        "identity",
        "late_promise",
        "same_environment",
        "global_fallback",
    ];
    let rows: Vec<_> = lines.collect();
    assert_eq!(
        rows.len(),
        names.len(),
        "fixture must contain exactly 20 rows"
    );
    for (row, expected_name) in rows.iter().zip(names) {
        let fields: Vec<_> = row.split('\t').collect();
        let [name, source, kind, data, visible, output] = fields.as_slice() else {
            panic!("invalid fixture row")
        };
        assert_eq!(*name, expected_name, "fixture row name/order");
        let visible = boolean_token(visible, "visibility");
        assert!(visible, "fixture visibility must be TRUE");
        let expected = match *kind {
            "integer" => Sexp::Integer(data.split(',').map(|s| Some(s.parse().unwrap())).collect()),
            "logical" => Sexp::Logical(
                data.split(',')
                    .map(|s| Some(boolean_token(s, "logical")))
                    .collect(),
            ),
            other => panic!("unexpected type {other}"),
        };
        let mut frontend = Frontend::default();
        let values = frontend
            .eval_source(source)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(values.last(), Some(&expected), "{name}");
        assert_eq!(frontend.runtime().is_visible(), visible, "{name}");
        let mut stdout = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.into_bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                OutputEvent::Stderr(text) => panic!("{name}: unexpected diagnostic {text}"),
            }
        }
        let actual = if stdout.is_empty() {
            "-".into()
        } else {
            stdout
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        assert_eq!(actual, *output, "{name}");
        println!("{name}: type/value/visibility/output PASS");
    }
}
