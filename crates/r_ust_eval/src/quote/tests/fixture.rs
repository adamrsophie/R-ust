use std::collections::HashMap;

use crate::test_support::fixture_json::hex;
use crate::{EvalError, Frontend, OutputEvent};

pub(super) enum Mapping {
    Direct,
    SdkNamespace,
    CatalogueBoundary,
    ReferenceOnly,
}
pub(super) enum Observation {
    Value(Box<[&'static str; 16]>),
    Error,
}
pub(super) struct Case {
    pub id: &'static str,
    pub setup: String,
    pub subject: String,
    pub after: String,
    pub mapping: Mapping,
    pub observation: Observation,
}

pub(super) fn decode(value: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(hex(value)?)?)
}

pub(super) fn quote_rows() -> Result<Vec<Case>, Box<dyn std::error::Error>> {
    rows(
        include_str!("../../../tests/fixtures/quote_runtime/cases.tsv"),
        include_str!("../../../tests/fixtures/quote_runtime/reference.tsv"),
        include_str!("../../../tests/fixtures/quote_runtime/mapping.tsv"),
    )
}

pub(super) fn rows(
    cases: &'static str,
    reference: &'static str,
    mapping: &'static str,
) -> Result<Vec<Case>, Box<dyn std::error::Error>> {
    let inputs: Vec<_> = cases.lines().skip(1).collect();
    let values: Vec<_> = reference.lines().skip(1).collect();
    let modes: Vec<_> = mapping.lines().skip(1).collect();
    assert_eq!(inputs.len(), values.len());
    assert_eq!(inputs.len(), modes.len());
    let mut ids = std::collections::HashSet::new();
    inputs
        .into_iter()
        .zip(values)
        .zip(modes)
        .map(|((input, value), mode)| {
            let fields: [&str; 6] = input
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "input columns")?;
            let observed: [&str; 16] = value
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "reference columns")?;
            let (id, mode) = mode.split_once('\t').ok_or("mapping columns")?;
            assert!(ids.insert(id), "duplicate row {id}");
            assert_eq!(fields[0], id);
            assert_eq!(observed[0], id);
            assert_eq!(fields[5], observed[12]);
            let mapping = match mode {
                "direct" => Mapping::Direct,
                "sdk-namespace" => Mapping::SdkNamespace,
                "catalogue-boundary" => Mapping::CatalogueBoundary,
                "reference-only" => Mapping::ReferenceOnly,
                _ => return Err("mapping kind".into()),
            };
            let observation = match observed[2] {
                "value" => Observation::Value(Box::new(observed)),
                "error" => Observation::Error,
                _ => return Err("observation kind".into()),
            };
            Ok(Case {
                id,
                setup: decode(fields[2])?,
                subject: decode(fields[3])?,
                after: decode(fields[4])?,
                mapping,
                observation,
            })
        })
        .collect()
}

pub(super) fn quote_errors() -> Result<HashMap<&'static str, String>, Box<dyn std::error::Error>> {
    errors(include_str!(
        "../../../tests/fixtures/quote_runtime/errors.tsv"
    ))
}

pub(super) fn errors(
    input: &'static str,
) -> Result<HashMap<&'static str, String>, Box<dyn std::error::Error>> {
    input
        .lines()
        .skip(1)
        .map(|line| {
            let [id, payload, _]: [&str; 3] = line
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "error columns")?;
            Ok((id, decode(payload)?))
        })
        .collect()
}

pub(super) fn payload(error: EvalError) -> String {
    match error {
        EvalError::Arity(message)
        | EvalError::Domain(message)
        | EvalError::Type(message)
        | EvalError::Stop { message, .. } => message,
        EvalError::MissingArgument(_) => error.to_string(),
        other => panic!("unexpected error {other:?}"),
    }
}

pub(super) fn output(frontend: &mut Frontend) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for event in frontend.runtime_mut().take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.push_str(&text),
            OutputEvent::StdoutBytes(bytes) => {
                stdout.push_str(std::str::from_utf8(&bytes).unwrap())
            }
            OutputEvent::Stderr(text) => stderr.push_str(&text),
        }
    }
    (stdout, stderr)
}
