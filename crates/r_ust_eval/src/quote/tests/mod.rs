mod accessor;
mod fixture;
mod sdk;

use crate::test_support::{fixture_json, fixture_node};
use crate::{class_value, format_sexp, mode_value, typeof_value, value_len, Frontend, Sexp};
use fixture::{decode, quote_errors as errors, quote_rows as rows, Case, Mapping, Observation};

#[test]
fn quote_matches_independent_r_trees_visibility_metadata_warnings_and_effects(
) -> Result<(), Box<dyn std::error::Error>> {
    let cases = rows()?;
    assert_eq!(cases.len(), 67);
    let payloads = errors()?;
    let mut counts = [0; 3];
    let mut failures = Vec::new();
    for case in cases {
        match case.mapping {
            Mapping::Direct => counts[0] += 1,
            Mapping::SdkNamespace => {
                counts[1] += 1;
                continue;
            }
            Mapping::ReferenceOnly => {
                counts[2] += 1;
                continue;
            }
            Mapping::CatalogueBoundary => panic!("unexpected quote catalogue mapping"),
        }
        let mut frontend = Frontend::default();
        frontend.eval_source(&case.setup)?;
        frontend.runtime_mut().take_output();
        let result = frontend.eval_source(&case.subject);
        match &case.observation {
            Observation::Value(fields) => {
                let values = match result {
                    Ok(values) => values,
                    Err(error) => {
                        failures.push(format!("{}: {error:?}", case.id));
                        continue;
                    }
                };
                let [actual] = values.as_slice() else {
                    panic!("{}: expected one result", case.id)
                };
                let node = fixture_node::node(&fixture_json::parse(&decode(fields[3])?)?)?;
                assert_eq!(actual, &node, "{} tree", case.id);
                assert_eq!(
                    frontend.runtime().visible,
                    fields[8] == "TRUE",
                    "{} visibility",
                    case.id
                );
                assert_eq!(typeof_value(actual), fields[4], "{} typeof", case.id);
                assert_eq!(
                    class_value(actual).unwrap(),
                    string(&decode(fields[5])?),
                    "{} class",
                    case.id
                );
                assert_eq!(mode_value(actual), fields[6], "{} mode", case.id);
                assert_eq!(
                    value_len(actual).unwrap().to_string(),
                    fields[7],
                    "{} length",
                    case.id
                );
                assert_eq!(
                    format_sexp(actual)?,
                    decode(fields[9])?,
                    "{} print",
                    case.id
                );
                frontend.eval_source(&case.after)?;
                let (output, warnings) = fixture::output(&mut frontend);
                assert_eq!(
                    output.strip_suffix('\n').unwrap_or(&output),
                    decode(fields[11])?,
                    "{} effects",
                    case.id
                );
                let conditions = decode(fields[10])?;
                let expected = conditions
                    .lines()
                    .collect::<Vec<_>>()
                    .chunks(3)
                    .map(|condition| format!("Warning: {}\n", condition[1]))
                    .collect::<String>();
                assert_eq!(warnings, expected, "{} warnings", case.id);
            }
            Observation::Error => {
                let error = result.expect_err(case.id);
                assert_eq!(
                    fixture::payload(error),
                    payloads[case.id],
                    "{} error",
                    case.id
                );
                let (output, warnings) = fixture::output(&mut frontend);
                assert!(output.is_empty(), "{} effects: {output}", case.id);
                assert!(warnings.is_empty(), "{} warnings: {warnings}", case.id);
            }
        }
    }
    assert_eq!(counts, [63, 2, 2]);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn string(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}

#[test]
fn namespace_missing_reference_rows_map_to_genuine_sdk_primitive_promises(
) -> Result<(), Box<dyn std::error::Error>> {
    let payloads = errors()?;
    let cases: Vec<Case> = rows()?
        .into_iter()
        .filter(|case| matches!(case.mapping, Mapping::SdkNamespace))
        .collect();
    assert_eq!(cases.len(), 2);
    for case in cases {
        let mut frontend = Frontend::default();
        frontend.eval_source(&case.setup)?;
        let call = Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![crate::CallArgument::positional(sdk::quote_call(
                Sexp::MissingValue,
            ))],
        };
        let error = frontend.runtime_mut().eval(&call).expect_err(case.id);
        assert_eq!(fixture::payload(error), payloads[case.id], "{}", case.id);
        let (output, warnings) = fixture::output(&mut frontend);
        let expected = if case.id == "missing_quote_function_cached" {
            "symbol \n"
        } else {
            ""
        };
        assert_eq!(output, expected, "{}", case.id);
        assert!(warnings.is_empty());
    }
    Ok(())
}
