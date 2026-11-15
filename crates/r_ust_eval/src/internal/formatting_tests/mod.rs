use crate::{Frontend, OutputEvent, Sexp};

mod coercion_edges;
mod correction_0;
mod correction_1;
mod correction_sdk;
mod dependency_0;
mod dependency_1;
mod dependency_wrappers;
mod fixtures;
mod heads_edges;
mod sdk;

struct Row {
    id: &'static str,
    source: &'static str,
    error: bool,
    stdout: &'static str,
    stderr: &'static str,
}

fn check(row: Row) {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("options(warn=1,keep.source=FALSE)")
        .unwrap();
    assert!(frontend.runtime.take_output().is_empty());
    let source = if row.error {
        format!(
            "tryCatch({{{}}},error=function(e)c(conditionMessage(e),class(e)))",
            row.source
        )
    } else {
        format!("{{{}}}", row.source)
    };
    let value = frontend
        .eval_source(&source)
        .unwrap_or_else(|error| panic!("{}: {error}", row.id))
        .pop()
        .expect("result");
    let Sexp::String(values) = value else {
        panic!("{} non-character {value:?}", row.id)
    };
    let mut lines = row.stdout.lines();
    assert_eq!(lines.next(), Some("D2FORMAT1"), "{} observer", row.id);
    assert_eq!(
        lines.next(),
        Some(if row.error { "error" } else { "value" })
    );
    if row.error {
        let message = decode(lines.next().expect("error message"));
        let mut expected = vec![Some(message)];
        expected.extend(
            lines
                .next()
                .expect("classes")
                .split(',')
                .map(|s| Some(s.into())),
        );
        assert_eq!(values, expected, "{} condition message/classes", row.id);
    } else {
        assert_eq!(
            frontend.runtime.is_visible(),
            lines.next() == Some("visible"),
            "{} visibility",
            row.id
        );
        let count: usize = lines.next().expect("length").parse().unwrap();
        let expected = lines
            .by_ref()
            .take(count)
            .map(|atom| {
                if atom == "N" {
                    None
                } else {
                    Some(decode(atom.strip_prefix('S').expect("string atom")))
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), count, "{} expected length", row.id);
        assert_eq!(values, expected, "{} value", row.id);
    }
    assert!(lines.next().is_none(), "{} trailing observer data", row.id);
    let (stdout, stderr) = streams(&mut frontend);
    assert!(stdout.is_empty(), "{} unexpected stdout: {stdout}", row.id);
    assert_eq!(stderr, row.stderr, "{} stderr", row.id);
    assert!(
        frontend.runtime.conditions.head.is_none(),
        "{} handler leak",
        row.id
    );
    assert!(
        frontend.runtime.conditions.restarts.is_none(),
        "{} restart leak",
        row.id
    );
    assert!(
        frontend.runtime.control.contexts.is_empty(),
        "{} control leak",
        row.id
    );
}

fn decode(value: &str) -> String {
    assert_eq!(value.len() % 2, 0);
    String::from_utf8(
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect(),
    )
    .unwrap()
}

fn streams(frontend: &mut Frontend) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for event in frontend.runtime.take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.push_str(&text),
            OutputEvent::Stderr(text) => stderr.push_str(&text),
            _ => panic!("unexpected output event"),
        }
    }
    (stdout, stderr)
}
