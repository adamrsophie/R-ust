use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn computed_function_positions_match_direct_r_reference() {
    for row in include_str!("fixtures/call-expression-r-4.6.0.tsv").lines() {
        let fields: [&str; 5] = row.split('\t').collect::<Vec<_>>().try_into().unwrap();
        let mut frontend = Frontend::default();
        let evaluation = frontend.eval_source(&format!("withVisible({{{}}})", fields[0]));
        match fields[1] {
            "OK" => {
                let values = evaluation.unwrap_or_else(|error| panic!("{}: {error:?}", fields[0]));
                let [Sexp::List(result)] = values.as_slice() else {
                    panic!("{}: {values:?}", fields[0])
                };
                assert_eq!(encode(&result[0].value), fields[2], "{}", fields[0]);
                assert_eq!(
                    result[1].value,
                    Sexp::Logical(vec![Some(fields[3] == "TRUE")]),
                    "{}",
                    fields[0]
                );
            }
            "ERR" => {
                let EvalError::Type(message) = evaluation.expect_err(fields[0]) else {
                    panic!("unexpected error for {}", fields[0])
                };
                assert_eq!(message, fields[2], "{}", fields[0]);
            }
            status => panic!("unexpected reference status {status}"),
        }
        let mut stdout = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                other => panic!("{}: unexpected output {other:?}", fields[0]),
            }
        }
        let hex = if stdout.is_empty() {
            "-".into()
        } else {
            stdout
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        assert_eq!(hex, fields[4], "{}", fields[0]);
    }
}

fn encode(value: &Sexp) -> String {
    match value {
        Sexp::Integer(values) => format!(
            "integer:{}",
            values
                .iter()
                .map(|v| v.unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",")
        ),
        Sexp::Real(values) => format!(
            "double:{}",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Sexp::Logical(values) => format!(
            "logical:{}",
            values
                .iter()
                .map(|v| if v.unwrap() { "TRUE" } else { "FALSE" })
                .collect::<Vec<_>>()
                .join(",")
        ),
        other => panic!("unexpected reference value {other:?}"),
    }
}
