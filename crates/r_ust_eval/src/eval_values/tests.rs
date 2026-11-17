use crate::{BinaryOp, EvalError, EvaluatedArgument, ListElement, RuntimeState, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector, ValueImport, ValueObject};

fn argument(value: Sexp) -> EvaluatedArgument {
    EvaluatedArgument { name: None, value }
}

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

#[test]
fn eval_typed_and_managed_expressions_use_selected_named_list_environment() -> Result<(), EvalError>
{
    let mut runtime = RuntimeState::default();
    let owner = runtime.global.clone();
    let expression = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![Sexp::Binary {
            op: BinaryOp::Add,
            left: Box::new(Sexp::Symbol("x".into())),
            right: Box::new(Sexp::Integer(vec![Some(2)])),
        }]),
        attributes: vec![attribute("note", Sexp::Symbol("never_evaluate".into()))],
    };
    let environment = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Integer(vec![Some(7)])]),
        attributes: vec![attribute("names", Sexp::String(vec![Some("x".into())]))],
    };
    for managed in [false, true] {
        let wrap = |value: &Sexp| -> Result<Sexp, EvalError> {
            Ok(if managed {
                Sexp::Managed(ValueObject::import(&owner, value, ValueImport::Evaluated)?)
            } else {
                value.clone()
            })
        };
        let value = runtime.internal_eval(&[
            argument(wrap(&expression)?),
            argument(wrap(&environment)?),
            argument(wrap(&Sexp::Environment(owner.clone()))?),
        ])?;
        assert_eq!(crate::numeric_values(&value)?, vec![9.0]);
    }
    assert!(owner.get_local("x")?.is_none());
    Ok(())
}

#[test]
fn eval_self_evaluating_managed_atomic_returns_original_transport() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let value = ValueObject::import(
        &runtime.global,
        &Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(7)]),
            attributes: vec![attribute("note", Sexp::Integer(vec![Some(1)]))],
        },
        ValueImport::Evaluated,
    )?;
    let output = runtime.internal_eval(&[
        argument(Sexp::Managed(value.clone())),
        argument(Sexp::Nil),
        argument(Sexp::Nil),
    ])?;
    let Sexp::Managed(output) = output else {
        panic!("original transport")
    };
    assert!(value.same_object(&output));
    Ok(())
}

#[test]
fn eval_managed_symbol_and_typed_numeric_environment_follow_existing_lookup(
) -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let owner = runtime.global.clone();
    owner.insert("x".into(), &Sexp::Integer(vec![Some(11)]))?;
    let environment = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(0)]),
        attributes: vec![],
    };
    for mode in [ValueImport::Code, ValueImport::Evaluated] {
        let symbol = ValueObject::import(&owner, &Sexp::Symbol("x".into()), mode)?;
        let output = runtime.internal_eval(&[
            argument(Sexp::Managed(symbol)),
            argument(environment.clone()),
            argument(Sexp::Nil),
        ])?;
        assert_eq!(output, Sexp::Integer(vec![Some(11)]));
    }
    Ok(())
}

#[test]
fn eval_bytecode_uses_selected_environment_in_both_execution_modes() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let selected = runtime.global.child()?;
    selected.insert("x".into(), &Sexp::Integer(vec![Some(17)]))?;
    let code = crate::bytecode::compile::expression(&selected, &Sexp::Symbol("x".into()))?
        .ok_or_else(|| EvalError::Type("expected supported symbol bytecode".into()))?;
    for enabled in [false, true] {
        runtime.set_bytecode_enabled(enabled);
        let output = runtime.internal_eval(&[
            argument(Sexp::Bytecode(code.clone())),
            argument(Sexp::Environment(selected.clone())),
            argument(Sexp::Nil),
        ])?;
        assert_eq!(output, Sexp::Integer(vec![Some(17)]));
    }
    Ok(())
}
