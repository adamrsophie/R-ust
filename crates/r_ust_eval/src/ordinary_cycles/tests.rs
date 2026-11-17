use super::check;
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, Environment, ListElement, ListObject, ValueImport, ValueObject};

#[test]
fn managed_explicit_list_cycle_is_rejected_without_recursing_forever() -> Result<(), EvalError> {
    let owner = Environment::global();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let value = Sexp::Managed(ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?);
    list.replace_shared(&[(0, value.clone())])?;
    assert_eq!(
        check(&value, "ordinary cycle guard"),
        Err(EvalError::NotImplemented("ordinary cycle guard"))
    );
    crate::character_attributes::validate(&value)?;
    Ok(())
}

#[test]
fn typed_aggregate_repeated_shared_children_are_not_cycles() -> Result<(), EvalError> {
    let owner = Environment::global();
    let list = ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    let value = ValueObject::import(&owner, &Sexp::ListObject(list), ValueImport::Explicit)?;
    let data = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Managed(value.clone()), Sexp::Managed(value)]),
        attributes: vec![],
    };
    check(&data, "ordinary cycle guard")?;
    Ok(())
}
