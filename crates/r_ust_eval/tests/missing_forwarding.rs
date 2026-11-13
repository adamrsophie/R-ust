use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn missing_preserves_direct_defaults_and_recursive_dot_resolution() {
    for row in include_str!("fixtures/missing-forwarding-r-4.6.0.tsv").lines() {
        let fields: [&str; 5] = row.split('\t').collect::<Vec<_>>().try_into().unwrap();
        let mut frontend = Frontend::default();

        let result = frontend
            .eval_source(&format!("withVisible({{{}}})", fields[0]))
            .unwrap_or_else(|error| panic!("{}: {error:?}", fields[0]));

        let [Sexp::List(result)] = result.as_slice() else {
            panic!("{}: {result:?}", fields[0])
        };
        let expected = fields[1]
            .strip_prefix("logical:")
            .expect("independent logical reference value")
            .split(',')
            .map(|value| Some(reference_boolean(value)))
            .collect();
        assert_eq!(result[0].value, Sexp::Logical(expected), "{}", fields[0]);
        assert_eq!(
            result[1].value,
            Sexp::Logical(vec![Some(reference_boolean(fields[2]))]),
            "{}",
            fields[0]
        );
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                OutputEvent::Stderr(text) => stderr.extend(text.bytes()),
            }
        }
        assert_eq!(hex(&stdout), fields[3], "{}", fields[0]);
        assert_eq!(hex(&stderr), fields[4], "{}", fields[0]);
    }
}

fn reference_boolean(value: &str) -> bool {
    match value {
        "TRUE" => true,
        "FALSE" => false,
        other => panic!("unexpected reference logical {other}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "-".into()
    } else {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
