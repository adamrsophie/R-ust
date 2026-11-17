use super::{attribute, check};
use crate::{EvalError, Sexp};
use r_ust_sexp::{
    AggregateVector, ClosureEnvironment, Environment, EvaluationOrigin, PromiseState, ValueImport,
    ValueObject,
};

#[test]
fn managed_print_mp4_plain_and_attributed_expression_and_symbol_cells_use_question_summary(
) -> Result<(), EvalError> {
    let expression = Sexp::Expression {
        elements: vec![crate::ListElement {
            name: None,
            value: Sexp::Symbol("x".into()),
        }],
        names_present: false,
    };
    let typed = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![Sexp::Symbol("x".into())]),
        attributes: vec![attribute("note", Sexp::Integer(vec![Some(1)]))],
    };
    check(
        "unknown_matrix",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![expression, typed, Sexp::Symbol("x".into())]),
            attributes: vec![attribute("dim", Sexp::Integer(vec![Some(1), Some(3)]))],
        },
    )
}

#[test]
fn managed_print_mp4_unknown_unclassed_cells_do_not_evaluate_bodies() -> Result<(), EvalError> {
    let expression = Sexp::Expression {
        elements: vec![crate::ListElement {
            name: None,
            value: Sexp::Symbol("x".into()),
        }],
        names_present: false,
    };
    let closure = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Symbol("UNFORCED".into())),
        environment: ClosureEnvironment::Base,
        attributes: vec![],
    };
    check(
        "unknown_mixed_matrix",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![
                expression,
                Sexp::Symbol("x".into()),
                Sexp::Environment(Environment::default()),
                closure,
            ]),
            attributes: vec![attribute("dim", Sexp::Integer(vec![Some(2), Some(2)]))],
        },
    )
}

#[test]
fn managed_print_mp4_promise_cell_uses_source_default_without_forcing() -> Result<(), EvalError> {
    let owner = Environment::default();
    let expression = Sexp::Symbol("UNFORCED".into());
    let state = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Promise {
            state: state.clone(),
        }]),
        attributes: vec![attribute("dim", Sexp::Integer(vec![Some(1), Some(1)]))],
    };
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        assert_eq!(super::super::format_sexp(value)?, "     [,1]\n[1,] ?   ");
    }
    assert!(state.environment()?.is_some());
    assert_eq!(state.expression()?, expression);
    Ok(())
}
