use crate::language_print::{format_deparse, format_language, LanguagePrintError, Profile};
use crate::{CallArgument, EvalError, RuntimeState, Sexp};
use r_ust_sexp::{EvaluationOrigin, PromiseState};

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values
            .into_iter()
            .map(|value| CallArgument { name: None, value })
            .collect(),
    }
}

fn profile(control: u32) -> Profile {
    Profile {
        cutoff: 60,
        backtick: true,
        control,
        lines: None,
        utf8: false,
    }
}

#[test]
fn correction_legacy_printer_retains_strict_shapes_and_ascii() {
    for value in [
        call("function", vec![Sexp::Nil, Sexp::Nil]),
        call("while", vec![Sexp::Nil, Sexp::Nil, Sexp::Nil]),
        call("(", vec![Sexp::Nil, Sexp::Nil]),
    ] {
        assert_eq!(
            format_language(&value),
            Err(LanguagePrintError::MalformedCall)
        );
    }
    let dollar = call(
        "$",
        vec![
            Sexp::Symbol("x".into()),
            Sexp::String(vec![Some("é".into())]),
        ],
    );
    assert_eq!(
        format_language(&dollar),
        Err(LanguagePrintError::UnsupportedText)
    );
}

#[test]
fn correction_source_enabled_string_remains_explicit_boundary() {
    let attributed = Sexp::AttributedCharacter {
        values: vec![Some("source".into())],
        attributes: vec![crate::ListElement {
            name: Some("class".into()),
            value: Sexp::String(vec![Some("source_text".into())]),
        }],
    };
    for source in [
        Sexp::String(vec![Some("source".into())]),
        Sexp::String(vec![]),
        Sexp::String(vec![None]),
        attributed.clone(),
        Sexp::AttributedCharacter {
            values: vec![Some("source".into())],
            attributes: vec![crate::ListElement {
                name: Some("dim".into()),
                value: Sexp::Integer(vec![Some(1), Some(1), Some(1)]),
            }],
        },
        matrix(Sexp::String(vec![Some("source".into())])),
        matrix(attributed.clone()),
        matrix(matrix(attributed)),
    ] {
        let value = call("function", vec![Sexp::Nil, Sexp::Nil, source]);
        assert!(matches!(
            format_deparse(&mut RuntimeState::default(), &value, profile(1089 | 8)),
            Err(EvalError::NotImplemented("deparse function source text"))
        ));
        assert_eq!(
            format_deparse(&mut RuntimeState::default(), &value, profile(1089)).unwrap(),
            (vec!["function() NULL".into()], false)
        );
    }
}

fn matrix(data: Sexp) -> Sexp {
    Sexp::Matrix {
        data: Box::new(data),
        nrow: 1,
        ncol: 1,
    }
}

#[test]
fn correction_source_noncharacter_storage_ignores_third_operand() {
    let factor = Sexp::Factor {
        codes: vec![Some(1)],
        levels: vec!["source".into()],
        ordered: false,
    };
    for source in [
        matrix(Sexp::Integer(vec![Some(1)])),
        matrix(Sexp::Real(vec![1.0])),
        matrix(Sexp::Logical(vec![Some(true)])),
        matrix(factor.clone()),
        matrix(Sexp::List(vec![crate::ListElement {
            name: None,
            value: Sexp::String(vec![Some("source".into())]),
        }])),
        factor,
        Sexp::Char(Some("source".into())),
        Sexp::Vector(vec![Sexp::String(vec![Some("source".into())])]),
    ] {
        let value = call("function", vec![Sexp::Nil, Sexp::Nil, source]);
        assert_eq!(
            format_deparse(&mut RuntimeState::default(), &value, profile(1089 | 8)).unwrap(),
            (vec!["function() NULL".into()], false)
        );
    }
}

#[test]
fn correction_unused_control_operand_does_not_force_promise() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let state = PromiseState::new(
        &Sexp::Symbol("UNFORCED".into()),
        &runtime.current,
        EvaluationOrigin::Global,
        true,
    )?;
    let value = call(
        "(",
        vec![
            Sexp::Integer(vec![Some(1)]),
            Sexp::Promise {
                state: state.clone(),
            },
        ],
    );
    assert_eq!(
        format_deparse(&mut runtime, &value, profile(1089))?,
        (vec!["(1L)".into()], false)
    );
    assert!(state.environment()?.is_some());
    assert!(runtime.take_output().is_empty());
    Ok(())
}
