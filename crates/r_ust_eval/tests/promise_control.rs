use std::collections::HashSet;

use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn decode_hex(text: &str) -> Vec<u8> {
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

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn condition(error: &EvalError) -> String {
    match error {
        EvalError::Control(message) | EvalError::Arity(message) | EvalError::Domain(message) => {
            message.clone()
        }
        other => other.to_string(),
    }
}

fn fingerprint(value: &Sexp) -> (String, String) {
    match value {
        Sexp::Nil => ("NULL".into(), "NULL".into()),
        Sexp::Integer(values) => (
            "integer".into(),
            format!(
                "integer|{}|{}",
                values.len(),
                values
                    .iter()
                    .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ),
        other => (format!("{:?}", other.sexptype()), format!("{other:?}")),
    }
}

fn run_fixture() {
    let mut seen = HashSet::new();
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in include_str!("fixtures/promise-control-r-4.6.0.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7, "{line}");
        let id: usize = fields[0].parse().unwrap();
        assert!((1..=45).contains(&id) && seen.insert(id), "{line}");
        let expected_error = decode_hex(fields[4]);
        let warnings = String::from_utf8(decode_hex(fields[5])).unwrap();
        let expected_output = decode_hex(fields[6]);
        assert!(matches!(fields[2], "error" | "integer" | "NULL"));
        assert_eq!(fields[2] == "error", !expected_error.is_empty());
        assert_eq!(fields[2] == "error", fields[3] == "-");
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)").unwrap();
        assert!(frontend.runtime_mut().take_output().is_empty());
        let result = frontend.eval_source(fields[1]);
        let (kind, value, error) = match &result {
            Ok(values) => {
                let (kind, value) = fingerprint(values.last().expect("fixture has a result"));
                (kind, value, Vec::new())
            }
            Err(error) => ("error".into(), "-".into(), condition(error).into_bytes()),
        };
        let mut output = Vec::new();
        let mut stderr = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => output.extend_from_slice(text.as_bytes()),
                OutputEvent::StdoutBytes(bytes) => output.extend(bytes),
                OutputEvent::Stderr(text) => stderr.extend_from_slice(text.as_bytes()),
            }
        }
        // The fixture stores raw warning conditions; the public sink adds this fixed prefix.
        let expected_stderr = warnings
            .lines()
            .map(|warning| format!("Warning: {warning}\n"))
            .collect::<String>()
            .into_bytes();
        let passed = kind == fields[2]
            && value == fields[3]
            && error == expected_error
            && output == expected_output
            && stderr == expected_stderr;
        println!(
            "ROW\t{id}\t{}\t{}\t{kind}\t{value}\t{}\t{}\t{}",
            if passed { "PASS" } else { "FAIL" },
            fields[1],
            hex(&error),
            hex(&stderr),
            hex(&output)
        );
        if !passed {
            failures.push(format!(
                "row {id}: {result:?}; expected {line}; stderr {}; stdout {}",
                hex(&stderr),
                hex(&output)
            ));
        }
        checked += 1;
    }
    assert_eq!(seen.len(), 45);
    assert_eq!(checked, 45);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_promise_control_fixture() {
    run_fixture();
}

#[test]
fn inactive_origins_and_cleanup() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source(
            "options(warn=1);maker<-function(x)function()x;h<-function()maker(return(42L));p<-h()",
        )
        .unwrap();
    for source in ["p()", "(function(){private<-7L;p();99L})()", "return(42L)"] {
        assert_eq!(
            condition(&frontend.eval_source(source).unwrap_err()),
            "no function to return from, jumping to top level",
            "{source}"
        );
        assert!(frontend.runtime().get("private").unwrap().is_none());
        assert_eq!(
            frontend.eval_source("42L").unwrap(),
            [Sexp::Integer(vec![Some(42)])]
        );
    }
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stderr(
            "Warning: restarting interrupted promise evaluation\n".into()
        )]
    );
    frontend
        .eval_source("p<-maker(break);for(i in 1:3)p()")
        .unwrap();
    for source in ["p()", "break", "next"] {
        assert_eq!(
            condition(&frontend.eval_source(source).unwrap_err()),
            "no loop for break/next, jumping to top level"
        );
        assert_eq!(
            frontend.eval_source("42L").unwrap(),
            [Sexp::Integer(vec![Some(42)])]
        );
    }
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stderr(
            "Warning: restarting interrupted promise evaluation\n".into()
        )]
    );
    assert!(matches!(
        frontend.eval_source("f<-function(x){private<-3L;x};f({cat('BEFORE');missing_name})"),
        Err(EvalError::UndefinedSymbol(name)) if name == "missing_name"
    ));
    assert!(frontend.runtime().get("private").unwrap().is_none());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("BEFORE".into())]
    );
    assert!(matches!(
        frontend.eval_source("f()"),
        Err(EvalError::MissingArgument(_))
    ));
    let values = frontend
        .eval_source("p<-maker({cat('ONCE');42L});list(p(),p())")
        .unwrap();
    let Sexp::List(values) = values.last().unwrap() else {
        panic!("expected list")
    };
    assert_eq!(values.len(), 2);
    assert!(values
        .iter()
        .all(|entry| entry.value == Sexp::Integer(vec![Some(42)])));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ONCE".into())]
    );
    let visible = frontend
        .eval_source_visible("(function(){f<-function(x)x;f(return(invisible(42L)));99L})()")
        .unwrap();
    assert!(visible.is_empty());
    assert_eq!(
        frontend.eval_source("42L").unwrap(),
        [Sexp::Integer(vec![Some(42)])]
    );
    assert!(frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
}

#[test]
fn transfers_cross_apply_callbacks_and_cached_closures() {
    let mut frontend = Frontend::default();
    for source in [
        "(function(){f<-function(x)lapply(1L,function(y)x);f(return(42L));99L})()",
        "f<-function(x)lapply(1L,function(y)x);for(i in 1:3){f(break);cat('NO')};42L",
        "f<-function(x)lapply(1L,function(y)x);for(i in 1:3){f(next);cat('NO')};42L",
    ] {
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Integer(vec![Some(42)])),
            "{source}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
    let values = frontend
        .eval_source("maker<-function(x)function()x;p<-maker({cat('ONCE');function()42L});p();p()")
        .unwrap();
    let value = values.last().unwrap();
    assert_eq!(value, &value.clone());
    assert!(format!("{value:?}").contains("Environment"));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ONCE".into())]
    );
    assert_eq!(
        frontend.eval_source("identical(p(),p())").unwrap(),
        [Sexp::Logical(vec![Some(true)])]
    );
}
