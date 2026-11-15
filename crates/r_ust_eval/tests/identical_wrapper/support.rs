use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn decode(value: &str) -> String {
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect::<Vec<_>>();
    String::from_utf8(bytes).unwrap()
}

fn strings(values: impl IntoIterator<Item = String>) -> Sexp {
    Sexp::String(values.into_iter().map(Some).collect())
}

fn value(fields: &[&str]) -> Sexp {
    let parts: Vec<_> = if fields[5].is_empty() {
        vec![]
    } else {
        fields[5].split(',').collect()
    };
    assert_eq!(parts.len(), fields[4].parse::<usize>().unwrap());
    match fields[2] {
        "logical" => Sexp::Logical(
            parts
                .iter()
                .map(|part| match *part {
                    "T" => Some(true),
                    "F" => Some(false),
                    "NA" => None,
                    _ => panic!("invalid logical fixture"),
                })
                .collect(),
        ),
        "integer" => Sexp::Integer(
            parts
                .iter()
                .map(|part| {
                    if *part == "NA" {
                        None
                    } else {
                        Some(part.parse().unwrap())
                    }
                })
                .collect(),
        ),
        "character" => Sexp::String(
            parts
                .iter()
                .map(|part| {
                    if *part == "NA" {
                        None
                    } else {
                        Some(decode(part.strip_prefix('x').unwrap()))
                    }
                })
                .collect(),
        ),
        other => panic!("unsupported fixture type {other}"),
    }
}

fn errors(frontend: &mut Frontend, fields: &[&str]) {
    for (expression, expected) in [
        (
            "class(captured)",
            strings(decode(fields[7]).split('|').map(str::to_owned)),
        ),
        (
            "deparse(conditionCall(captured))",
            strings(decode(fields[8]).split('\n').map(str::to_owned)),
        ),
        ("conditionMessage(captured)", strings([decode(fields[9])])),
    ] {
        assert_eq!(
            frontend.eval_source(expression).unwrap().pop(),
            Some(expected),
            "{}: {expression}",
            fields[0]
        );
    }
}

pub(super) fn check(id: &str, source: &str) {
    let disabled = std::env::var("R_DISABLE_BYTECODE").ok().as_deref() == Some("1");
    let fixture = if disabled {
        include_str!("../fixtures/identical-wrapper-disabled-1.tsv")
    } else {
        include_str!("../fixtures/identical-wrapper-disabled-0.tsv")
    };
    let fields: Vec<_> = fixture
        .lines()
        .find(|line| line.split('\t').next() == Some(id))
        .unwrap()
        .split('\t')
        .collect();
    assert_eq!(fields.len(), 10);
    let mut frontend = Frontend::default();
    match fields[1] {
        "VALUE" => {
            let result = frontend.eval_source(source).unwrap().pop().unwrap();
            assert_eq!(result, value(&fields), "{id}");
            assert_eq!(
                frontend.runtime().is_visible(),
                fields[3] == "TRUE",
                "{id}: visibility"
            );
        }
        "ERROR" => {
            frontend
                .eval_source(&format!(
                    "captured<-tryCatch({{{source}}},error=function(e)e)"
                ))
                .unwrap();
            errors(&mut frontend, &fields);
        }
        other => panic!("unexpected fixture status {other}"),
    }
    let mut stdout = Vec::new();
    let mut stderr = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.extend_from_slice(text.as_bytes()),
            OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
            OutputEvent::Stderr(text) => stderr.push_str(&text),
        }
    }
    assert_eq!(stdout, decode(fields[6]).as_bytes(), "{id}: effects");
    assert!(stderr.is_empty(), "{id}: unexpected stderr {stderr:?}");
}
