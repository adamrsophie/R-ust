use r_ust_eval::{Frontend, OutputEvent};
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
        Sexp::String(values) => (
            "character",
            values
                .iter()
                .map(|value| {
                    value
                        .as_ref()
                        .map_or_else(|| "NA".into(), |value| hex(value.as_bytes()))
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
        other => panic!("unexpected boundary value: {other:?}"),
    };
    (
        kind,
        format!("{kind}|{}|{}", values.len(), values.join(",")),
    )
}

pub fn check(owner: &str, expected_ids: &[&str]) {
    let mut lines = include_str!("../fixtures/wrapper-control-r-4.6.0.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("owner\tid\tsource\ttype\tvalue\tvisible\tstdout_hex\tstderr_hex")
    );
    let rows: Vec<Vec<_>> = lines.map(|line| line.split('\t').collect()).collect();
    assert_eq!(rows.len(), 37);
    let mut unique = std::collections::HashSet::new();
    for row in &rows {
        assert_eq!(row.len(), 8);
        assert!(unique.insert((row[0], row[1])));
        assert!(matches!(row[3], "character" | "integer"));
        assert!(matches!(row[5], "TRUE" | "FALSE"));
        assert!(row.iter().all(|field| !field.is_empty()));
    }
    let rows: Vec<_> = rows.into_iter().filter(|row| row[0] == owner).collect();
    assert_eq!(rows.len(), expected_ids.len());
    let mut failures = Vec::new();
    for (row, id) in rows.into_iter().zip(expected_ids) {
        assert_eq!(row[1], *id);
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(row[2]);
        let visible = if frontend.runtime().is_visible() {
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
            Ok(values) => canonical(values.last().expect("boundary result")),
            Err(error) => ("error", hex(error.to_string().as_bytes())),
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
            "CONTROL\t{owner}\t{id}\t{}\t{}",
            if passed { "PASS" } else { "FAIL" },
            actual.join("\t")
        );
        if !passed {
            failures.push(format!(
                "{owner}/{id}: {result:?}; actual={actual:?}; expected={:?}",
                &row[3..]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
