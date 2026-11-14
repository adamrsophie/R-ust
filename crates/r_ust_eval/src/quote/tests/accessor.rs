use super::fixture::{self, decode, Mapping, Observation};
use crate::test_support::{fixture_json, fixture_node};
use crate::{
    class_value, format_sexp, mode_value, typeof_value, value_len, EvalError, Frontend, Sexp,
};

#[test]
fn primitive_accessor_matches_r_identity_tags_forcing_and_validation(
) -> Result<(), Box<dyn std::error::Error>> {
    let cases = fixture::rows(
        include_str!("../../../tests/fixtures/primitive_accessor/cases.tsv"),
        include_str!("../../../tests/fixtures/primitive_accessor/reference.tsv"),
        include_str!("../../../tests/fixtures/primitive_accessor/mapping.tsv"),
    )?;
    let reference: Vec<_> =
        include_str!("../../../tests/fixtures/primitive_accessor/reference.tsv")
            .lines()
            .skip(1)
            .collect();
    let payloads = fixture::errors(include_str!(
        "../../../tests/fixtures/primitive_accessor/errors.tsv"
    ))?;
    assert_eq!(cases.len(), 62);
    let mut counts = [0; 3];
    let mut failures = Vec::new();
    for (case, raw) in cases.into_iter().zip(reference) {
        let fields: Vec<_> = raw.split('\t').collect();
        match case.mapping {
            Mapping::ReferenceOnly => {
                counts[2] += 1;
                continue;
            }
            Mapping::SdkNamespace => panic!("unexpected accessor namespace mapping"),
            Mapping::Direct => counts[0] += 1,
            Mapping::CatalogueBoundary => {
                counts[1] += 1;
                let result = Frontend::default().eval_source(&case.subject);
                match result {
                    Err(EvalError::NotImplemented("primitive catalogue entry")) => (),
                    Ok(values) => {
                        let [Sexp::Primitive(id)] = values.as_slice() else {
                            panic!("{}: expected primitive", case.id)
                        };
                        let json = fixture_json::parse(&decode(fields[3])?)?;
                        let [_, name] = fixture_json::array(&json)? else {
                            panic!("primitive node")
                        };
                        assert_eq!(id.name(), fixture_json::text(name)?);
                    }
                    other => panic!("{}: valid R primitive misdiagnosed: {other:?}", case.id),
                }
                continue;
            }
        }
        let mut frontend = Frontend::default();
        frontend.eval_source(&case.setup)?;
        frontend.runtime_mut().take_output();
        let result = frontend.eval_source(&case.subject);
        match case.observation {
            Observation::Value(observed) => {
                let values = match result {
                    Ok(values) => values,
                    Err(error) => {
                        failures.push(format!("{}: {error:?}", case.id));
                        continue;
                    }
                };
                let [actual] = values.as_slice() else {
                    panic!("{} result", case.id)
                };
                let expected = fixture_node::node(&fixture_json::parse(&decode(observed[3])?)?)?;
                assert_eq!(actual, &expected, "{} identity", case.id);
                assert_eq!(
                    frontend.runtime().visible,
                    observed[8] == "TRUE",
                    "{} visibility",
                    case.id
                );
                assert_eq!(typeof_value(actual), observed[4], "{} typeof", case.id);
                assert_eq!(
                    class_value(actual).unwrap(),
                    super::string(&decode(observed[5])?),
                    "{} class",
                    case.id
                );
                assert_eq!(mode_value(actual), observed[6], "{} mode", case.id);
                assert_eq!(
                    value_len(actual).unwrap().to_string(),
                    observed[7],
                    "{} length",
                    case.id
                );
                assert_eq!(
                    format_sexp(actual)?,
                    decode(observed[9])?,
                    "{} print",
                    case.id
                );
                frontend.eval_source(&case.after)?;
                let (stdout, stderr) = fixture::output(&mut frontend);
                assert_eq!(
                    stdout.strip_suffix('\n').unwrap_or(&stdout),
                    decode(observed[11])?,
                    "{} effects",
                    case.id
                );
                assert!(stderr.is_empty(), "{}: {stderr}", case.id);
            }
            Observation::Error => {
                assert_eq!(
                    fixture::payload(result.expect_err(case.id)),
                    payloads[case.id],
                    "{} error",
                    case.id
                );
                let (stdout, stderr) = fixture::output(&mut frontend);
                assert_eq!(stdout, decode(fields[13])?, "{} effects", case.id);
                assert!(stderr.is_empty(), "{}: {stderr}", case.id);
            }
        }
    }
    assert_eq!(counts, [54, 4, 4]);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
