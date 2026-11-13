use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "-".into()
    } else {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

fn canonical(value: &Sexp) -> (&'static str, String) {
    let (kind, values): (_, Vec<String>) = match value {
        Sexp::Nil => ("NULL", vec![]),
        Sexp::String(values) => (
            "character",
            values
                .iter()
                .map(|value| match value {
                    None => "NA".into(),
                    Some(value) if value.is_empty() => "x".into(),
                    Some(value) => format!("x{}", hex(value.as_bytes())),
                })
                .collect(),
        ),
        Sexp::Integer(values) => (
            "integer",
            values
                .iter()
                .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                .collect(),
        ),
        Sexp::Logical(values) => (
            "logical",
            values
                .iter()
                .map(|value| {
                    match value {
                        None => "NA",
                        Some(true) => "T",
                        Some(false) => "F",
                    }
                    .into()
                })
                .collect(),
        ),
        other => panic!("unexpected oracle result: {other:?}"),
    };
    (
        kind,
        format!("{kind}|{}|{}", values.len(), values.join(",")),
    )
}

fn condition(error: &EvalError) -> String {
    match error {
        EvalError::Arity(message) | EvalError::Control(message) => message.clone(),
        other => other.to_string(),
    }
}

pub fn check(owner: &str) {
    let mut lines = include_str!("../fixtures/evaluation-identity-force-r-4.6.0.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("owner\tid\tsource\ttype\tvalue\tvisible\tstdout_hex\twarnings_hex")
    );
    let rows: Vec<Vec<_>> = lines.map(|line| line.split('\t').collect()).collect();
    assert_eq!(rows.len(), 104);
    let mut unique = std::collections::HashSet::new();
    for row in &rows {
        assert_eq!(row.len(), 8);
        assert!(unique.insert((row[0], row[1])));
        assert!(matches!(row[0], "identity" | "force"));
        assert!(matches!(
            row[3],
            "NULL" | "character" | "integer" | "logical" | "error"
        ));
        assert!(matches!(row[5], "TRUE" | "FALSE" | "-"));
        assert_eq!(row[5] == "-", row[3] == "error");
        assert_eq!(row[7], "-");
        assert!(row.iter().all(|field| !field.is_empty()));
    }
    let rows: Vec<_> = rows.into_iter().filter(|row| row[0] == owner).collect();
    assert_eq!(rows.len(), 52);
    let mut failures = Vec::new();
    for row in rows {
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(row[2]);
        let visible = if result.is_err() {
            "-"
        } else if frontend.runtime().is_visible() {
            "TRUE"
        } else {
            "FALSE"
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.as_bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                OutputEvent::Stderr(text) => stderr.extend(text.as_bytes()),
            }
        }
        let (kind, value) = match &result {
            Ok(values) => canonical(values.last().expect("oracle result")),
            Err(error) => ("error", hex(condition(error).as_bytes())),
        };
        let actual = [
            kind.into(),
            value,
            visible.into(),
            hex(&stdout),
            hex(&stderr),
        ];
        let passed = actual
            .iter()
            .map(String::as_str)
            .eq(row[3..].iter().copied());
        println!(
            "EVALUATION\t{owner}\t{}\t{}\t{}",
            row[1],
            if passed { "PASS" } else { "FAIL" },
            actual.join("\t")
        );
        if !passed {
            failures.push(format!(
                "{owner}/{}: {result:?}; actual={actual:?}; expected={:?}",
                row[1],
                &row[3..]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
