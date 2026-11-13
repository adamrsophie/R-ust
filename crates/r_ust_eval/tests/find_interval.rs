use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

#[test]
fn find_interval_matches_r_4_6_boundaries_flags_and_errors() {
    let fixture = include_str!("fixtures/find-interval-r-4.6.0.tsv");
    let mut lines = fixture.lines();
    assert_eq!(
        lines.next(),
        Some("# R version 4.6.0 (2026-04-24) findInterval C locale")
    );
    assert_eq!(lines.next(), Some("id\tsource\tkind\tdata"));
    let mut count = 0;
    let mut failures = Vec::new();
    for (index, line) in lines.enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(matches!(fields[2], "integer" | "logical" | "error"));
        assert!(!fields[1].is_empty() && !fields[3].is_empty());
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[1]);
        let (kind, data) = match &result {
            Ok(values) => {
                if let Some(Sexp::Logical(values)) = values.last() {
                    let actual = values
                        .iter()
                        .map(|v| match v {
                            Some(true) => "TRUE",
                            Some(false) => "FALSE",
                            None => "NA",
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    assert_eq!(fields[2], "logical");
                    assert_eq!(actual, fields[3], "{}", fields[1]);
                    assert!(frontend.runtime_mut().take_output().is_empty());
                    println!("ROW\t{}\tPASS\t{}\tlogical\t{actual}", fields[0], fields[1]);
                    count += 1;
                    continue;
                }
                let Some(Sexp::Integer(values)) = values.last() else {
                    panic!("expected integer result: {result:?}");
                };
                let text = if values.is_empty() {
                    "-".into()
                } else {
                    values
                        .iter()
                        .map(|v| v.map_or_else(|| "NA".into(), |v| v.to_string()))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                ("integer", text)
            }
            Err(error) => (
                "error",
                match error {
                    EvalError::Domain(message)
                    | EvalError::Arity(message)
                    | EvalError::Type(message)
                    | EvalError::Stop { message, .. } => message.clone(),
                    other => other.to_string(),
                },
            ),
        };
        let output = frontend.runtime_mut().take_output();
        let passed = kind == fields[2] && data == fields[3] && output.is_empty();
        println!(
            "ROW\t{}\t{}\t{}\t{kind}\t{data}",
            fields[0],
            if passed { "PASS" } else { "FAIL" },
            fields[1]
        );
        if !passed {
            failures.push(format!("{line} => {result:?}; output {output:?}"));
        }
        count += 1;
    }
    assert_eq!(count, 127);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn invalid_unchecked_breaks_return_a_typed_error() {
    for source in [
        "findInterval(1,c(2,0),checkSorted=FALSE)",
        "findInterval(1,c(0,NA),checkSorted=FALSE)",
    ] {
        assert!(
            matches!(Frontend::default().eval_source(source),Err(EvalError::Domain(message))
            if message == "'vec' must be sorted non-decreasingly and not contain NAs")
        );
    }
}
