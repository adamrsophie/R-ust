use super::managed;
use crate::coercion::{lists, strings::BlankPolicy, warnings::Warnings};
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{is_na_real, AggregateVector, Environment, EvaluationOrigin, Force, PromiseState};

fn forms(owner: &Environment, values: Vec<Sexp>) -> Result<Vec<Sexp>, EvalError> {
    let expression = Sexp::Expression {
        elements: values
            .iter()
            .cloned()
            .map(|value| ListElement { name: None, value })
            .collect(),
        names_present: false,
    };
    let attributed = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(values),
        attributes: vec![],
    };
    Ok(vec![
        expression.clone(),
        attributed.clone(),
        managed(owner, &expression)?,
        managed(owner, &attributed)?,
    ])
}

#[test]
fn expression_children_use_length_policy_without_converting_contents() -> Result<(), EvalError> {
    let owner = Environment::global();
    for size in 0..=2 {
        for expression in forms(&owner, vec![Sexp::Integer(vec![Some(1)]); size])? {
            assert_eq!(lists::vectorizable(&expression)?, size <= 1);
            for integer in [true, false] {
                let direct = lists::convert(&expression, (integer, BlankPolicy::C), |_| {
                    panic!("direct expression rejection must not emit coercion warnings")
                });
                assert_eq!(
                    direct.unwrap_err().to_string(),
                    format!(
                        "type error: 'list' object cannot be coerced to type '{}'",
                        if integer { "integer" } else { "double" }
                    )
                );
                let outer = Sexp::Vector(vec![expression.clone()]);
                for input in [outer.clone(), managed(&owner, &outer)?] {
                    let mut warnings = Vec::new();
                    let result = lists::convert(&input, (integer, BlankPolicy::C), |warning| {
                        warnings.push(warning);
                        Ok(())
                    });
                    if size == 2 {
                        assert_eq!(
                            result.unwrap_err().to_string(),
                            format!(
                                "type error: 'list' object cannot be coerced to type '{}'",
                                if integer { "integer" } else { "double" }
                            )
                        );
                        assert!(warnings.is_empty());
                    } else {
                        assert_eq!(warnings, [Warnings::default()]);
                        if integer {
                            assert_eq!(result?, Sexp::Integer(vec![None]));
                        } else {
                            let Sexp::Real(values) = result? else {
                                panic!("expected real storage")
                            };
                            assert_eq!(values.len(), 1);
                            assert!(is_na_real(values[0]));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn expression_child_coercion_does_not_force_a_promise() -> Result<(), EvalError> {
    let owner = Environment::global();
    let state = PromiseState::new(
        &Sexp::Symbol("must_not_run".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    for expression in forms(
        &owner,
        vec![Sexp::Promise {
            state: state.clone(),
        }],
    )? {
        let outer = Sexp::Vector(vec![expression]);
        for input in [outer.clone(), managed(&owner, &outer)?] {
            let result = lists::convert(&input, (true, BlankPolicy::C), |warning| {
                assert_eq!(warning, Warnings::default());
                Ok(())
            })?;
            assert_eq!(result, Sexp::Integer(vec![None]));
        }
    }
    assert!(matches!(state.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
