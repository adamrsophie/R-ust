use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{
    AggregateVector, Environment, EvaluationOrigin, ListObject, PromiseState, ValueImport,
    ValueObject,
};

#[test]
fn managed_print_shared_children_are_not_cycles_and_keep_identity() -> Result<(), EvalError> {
    let owner = Environment::default();
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(7)]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![child.transport()?, child.transport()?]),
        attributes: vec![],
    };
    let root = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = child.semantic_reference_count()?;
    let descriptor = root.borrow()?;
    drop(root);
    drop(owner);
    assert_eq!(
        super::super::format_sexp(descriptor.value())?,
        "[[1]]\n[1] 7\n\n[[2]]\n[1] 7\n"
    );
    assert_eq!(
        super::support::deparse(descriptor.value(), 1093, false)?,
        "list(7L, 7L)\n"
    );
    let Sexp::AttributedAggregate { values, .. } = descriptor.value() else {
        panic!("descriptor")
    };
    for value in values.values() {
        let Sexp::Managed(value) = value else {
            panic!("transport")
        };
        assert!(child.same_object(value));
    }
    assert_eq!(child.semantic_reference_count()?, before);
    Ok(())
}

#[test]
fn managed_print_cycle_reports_error_and_shared_graph_remains_readable() -> Result<(), EvalError> {
    let owner = Environment::default();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let root = ValueObject::import(
        &owner,
        &Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![Sexp::ListObject(list.clone())]),
            attributes: vec![],
        },
        ValueImport::Evaluated,
    )?;
    list.replace_shared(&[(0, root.transport()?)])?;
    assert_eq!(
        super::super::format_sexp(&root.transport()?),
        Err(EvalError::NotImplemented("cyclic ordinary value printing"))
    );
    assert_eq!(
        super::support::deparse(&root.transport()?, 1093, false),
        Err(EvalError::NotImplemented("deparse cyclic graph value"))
    );
    list.replace_shared(&[(0, Sexp::Nil)])?;
    assert_eq!(
        super::support::deparse(&root.transport()?, 1093, false)?,
        "list(list(NULL))\n"
    );
    Ok(())
}

#[test]
fn managed_print_does_not_force_promise_children() -> Result<(), EvalError> {
    let owner = Environment::default();
    let state = PromiseState::new(
        &Sexp::Symbol("UNFORCED".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let root = ValueObject::import(
        &owner,
        &Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![Sexp::Promise {
                state: state.clone(),
            }]),
            attributes: vec![],
        },
        ValueImport::Evaluated,
    )?;
    assert_eq!(
        super::super::format_sexp(&root.transport()?)?,
        "[[1]]\n<promise>\n"
    );
    assert!(state.environment()?.is_some());
    assert_eq!(state.expression()?, Sexp::Symbol("UNFORCED".into()));
    Ok(())
}
