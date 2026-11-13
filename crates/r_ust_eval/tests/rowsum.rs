use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{is_na_real, Sexp};

fn atomic(value: &Sexp) -> (String, String) {
    let (kind, values): (_, Vec<String>) = match value {
        Sexp::Logical(v) if v.is_empty() => ("logical", Vec::new()),
        Sexp::String(v) if v.is_empty() => ("character", Vec::new()),
        Sexp::List(v) if v.is_empty() => ("list", Vec::new()),
        Sexp::Integer(v) => (
            "integer",
            v.iter()
                .map(|v| v.map_or("NA".into(), |v| v.to_string()))
                .collect(),
        ),
        Sexp::Real(v) => (
            "double",
            v.iter()
                .map(|v| {
                    if is_na_real(*v) {
                        "NA".into()
                    } else if v.is_nan() {
                        "NaN".into()
                    } else {
                        v.to_string()
                    }
                })
                .collect(),
        ),
        other => panic!("unexpected result {other:?}"),
    };
    (kind.into(), values.join(","))
}

fn canonical(value: &Sexp) -> String {
    match value {
        Sexp::Matrix { data, nrow, ncol } => {
            let (kind, values) = atomic(data);
            format!("{kind}:{nrow},{ncol}:{values}")
        }
        Sexp::DataFrame { columns, nrow } => format!(
            "df:{nrow}:{}",
            columns
                .iter()
                .map(|column| {
                    let (kind, values) = atomic(&column.value);
                    format!("{kind}:{values}")
                })
                .collect::<Vec<_>>()
                .join(";")
        ),
        other => panic!("unexpected result {other:?}"),
    }
}

#[test]
fn rowsum_matches_r_values_warnings_errors_and_forcing() {
    let fixture = include_str!("fixtures/rowsum-r-4.6.0.tsv");
    let mut lines = fixture.lines();
    assert_eq!(lines.next(), Some("id\tsource\tvalue\terror\twarning"));
    let lines: Vec<_> = lines.collect();
    assert_eq!(lines.len(), 44);
    let mut failures = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(fields.iter().all(|field| !field.is_empty()));
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[1]);
        let (actual, error) = match result {
            Ok(values) => (canonical(values.last().unwrap()), "-".into()),
            Err(
                EvalError::Type(message) | EvalError::Arity(message) | EvalError::Domain(message),
            ) => ("-".into(), message),
            Err(other) => ("-".into(), other.to_string()),
        };
        let warnings = frontend
            .runtime_mut()
            .take_output()
            .into_iter()
            .map(|event| match event {
                OutputEvent::Stderr(message) => message,
                other => panic!("unexpected output {other:?}"),
            })
            .collect::<String>();
        let expected_warning = if fields[4] == "-" {
            String::new()
        } else {
            format!("Warning message:\n{} \n", fields[4])
        };
        if actual != fields[2] || error != fields[3] || warnings != expected_warning {
            failures.push(format!(
                "{}: {}: value={actual}, error={error}, warnings={warnings:?}; expected {:?}",
                fields[0],
                fields[1],
                &fields[2..]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[path = "support/wrapper_control.rs"]
mod wrapper_control;

#[test]
fn rowsum_preserves_nested_control_and_visibility() {
    wrapper_control::check(
        "rowsum",
        &[
            "generic_return",
            "default_group_invisible_return",
            "group_break",
            "na_rm_next",
        ],
    );
}
