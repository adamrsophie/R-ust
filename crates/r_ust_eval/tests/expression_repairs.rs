use r_ust_eval::{Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{
    canonical_actual_call, canonical_language, CallArgument, LanguageError, ListElement,
    PrimitiveId, Sexp,
};

#[path = "expression_repairs/fixtures.rs"]
mod fixtures;
#[path = "expression_repairs/sdk.rs"]
mod sdk;

#[test]
fn repaired_capture_and_is_vector_match_independent_r_observations(
) -> Result<(), Box<dyn std::error::Error>> {
    for row in fixtures::rows() {
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1,keep.source=FALSE)")?;
        frontend.eval_source(row.setup)?;
        let actual = frontend.eval_source(row.source)?.pop().ok_or("result")?;
        assert_eq!(actual, row.expected, "{}", row.id);
        assert_eq!(
            frontend.runtime_mut().is_visible(),
            row.visible,
            "{}",
            row.id
        );
        let output = frontend.runtime_mut().take_output();
        let warnings = output
            .iter()
            .filter_map(|event| {
                if let OutputEvent::Stderr(text) = event {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<String>();
        if row.warning.is_empty() {
            assert!(warnings.is_empty(), "{}: {warnings}", row.id);
        } else {
            assert!(warnings.contains(row.warning), "{}: {warnings}", row.id);
        }
        let stdout = output
            .into_iter()
            .filter_map(|event| {
                if let OutputEvent::Stdout(text) = event {
                    Some(text)
                } else {
                    None
                }
            })
            .collect::<String>();
        assert_eq!(stdout, row.stdout, "{}", row.id);
    }
    Ok(())
}

#[test]
fn invalid_calls_fail_without_forcing_later_arguments() {
    for row in fixtures::errors() {
        let mut frontend = Frontend::default();
        frontend.eval_source(row.setup).unwrap();
        assert!(frontend.eval_source(row.source).is_err(), "{}", row.id);
        let stdout = frontend
            .runtime_mut()
            .take_output()
            .into_iter()
            .filter_map(|event| {
                if let OutputEvent::Stdout(text) = event {
                    Some(text)
                } else {
                    None
                }
            })
            .collect::<String>();
        assert_eq!(stdout, row.stdout, "{}", row.id);
    }
}
