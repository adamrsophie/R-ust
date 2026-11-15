use super::*;
use crate::{EvaluationOrigin, ListElement, Sexp};

fn list(owner: &Environment, values: Vec<Sexp>) -> Result<ListObject, GraphError> {
    ListObject::from_value(
        owner,
        &Sexp::List(
            values
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
        ),
    )
}

fn count(value: &ListObject, expected: usize) -> Result<(), GraphError> {
    assert_eq!(value.semantic_reference_count()?, expected);
    Ok(())
}

#[test]
fn semantic_owners_are_bindings_not_handle_clones_or_exports() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = list(&owner, vec![Sexp::Nil])?;
    let handle = value.clone();
    count(&value, 0)?;
    owner.insert("x".into(), &Sexp::ListObject(value.clone()))?;
    let snapshot = owner.get_local("x")?;
    count(&value, 1)?;
    owner.insert("alias".into(), &Sexp::ListObject(value.clone()))?;
    count(&value, 2)?;
    owner.insert("alias".into(), &Sexp::Nil)?;
    count(&value, 1)?;
    drop(snapshot);
    drop(handle);
    count(&value, 1)?;
    owner.insert("x".into(), &Sexp::Nil)?;
    count(&value, 0)
}

#[test]
fn repeated_slots_shallow_copy_and_collection_preserve_reference_history() -> Result<(), GraphError>
{
    let owner = Environment::global();
    let child = list(&owner, vec![])?;
    let outer = list(
        &owner,
        vec![
            Sexp::ListObject(child.clone()),
            Sexp::ListObject(child.clone()),
        ],
    )?;
    count(&child, 2)?;
    let copy = outer.shallow_copy()?;
    count(&child, 4)?;
    assert!(!copy.same_object(&outer));
    let Sexp::ListObject(selected) = copy.element(0)? else {
        panic!("child identity lost")
    };
    assert!(selected.same_object(&child));
    drop(copy);
    count(&child, 4)?;
    outer.replace_shared(&[(0, Sexp::Nil)])?;
    count(&child, 3)?;
    drop(outer);
    count(&child, 3)
}

#[test]
fn ordinary_cache_counts_and_replacement_cache_only_traces() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = list(&owner, vec![])?;
    let ordinary = PromiseState::new(&Sexp::Nil, &owner, EvaluationOrigin::Global, true)?;
    let replacement = PromiseState::new_replacement(
        &Sexp::Nil,
        &owner,
        PromiseMetadata {
            origin: EvaluationOrigin::Global,
            supplied: true,
        },
    )?;
    for promise in [&ordinary, &replacement] {
        let Force::Evaluate(lease) = promise.begin_force()? else {
            panic!("new promise cached")
        };
        lease.complete(&Sexp::ListObject(value.clone()))?;
    }
    count(&value, 1)?;
    drop(ordinary);
    count(&value, 1)?;
    let id = value.0.id();
    let heap = value.0.heap()?.clone();
    drop(value);
    assert!(heap.lock().node(id).is_ok());
    drop(replacement);
    assert_eq!(heap.lock().node(id).err(), Some(GraphError::StaleNode));
    Ok(())
}

#[test]
fn external_retention_lease_counts_once_and_releases() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = list(&owner, vec![])?;
    let lease = owner.retain_value(&Sexp::ListObject(value.clone()))?;
    let held = lease.clone();
    count(&value, 1)?;
    drop(lease);
    count(&value, 1)?;
    drop(held);
    count(&value, 0)
}

#[test]
fn saturated_reference_count_stays_saturated_after_alias_release() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = list(&owner, vec![])?;
    let aliases = list(
        &owner,
        vec![Sexp::ListObject(value.clone()); usize::from(u16::MAX)],
    )?;
    count(&value, usize::from(u16::MAX))?;
    drop(aliases);
    assert_eq!(value.semantic_reference_count()?, usize::from(u16::MAX));
    assert_eq!(
        value.0.heap()?.lock().audit_reference_count(value.0.id())?,
        0
    );
    Ok(())
}

#[test]
fn ordered_slot_updates_preserve_saturation_history() -> Result<(), GraphError> {
    for saturates in [false, true] {
        let owner = Environment::global();
        let value = list(&owner, vec![])?;
        let aliases = list(
            &owner,
            vec![Sexp::ListObject(value.clone()); usize::from(u16::MAX) - 2],
        )?;
        let slots = list(&owner, vec![Sexp::Nil, Sexp::ListObject(value.clone())])?;
        count(&value, usize::from(u16::MAX) - 1)?;
        slots.replace_shared(&[(1, Sexp::ListObject(value.clone()))])?;
        count(&value, usize::from(u16::MAX) - 1)?;
        let updates = if saturates {
            vec![(0, Sexp::ListObject(value.clone())), (1, Sexp::Nil)]
        } else {
            vec![(1, Sexp::Nil), (0, Sexp::ListObject(value.clone()))]
        };
        slots.replace_shared(&updates)?;
        assert_eq!(
            value.semantic_reference_count()?,
            usize::from(u16::MAX) - usize::from(!saturates)
        );
        assert_eq!(
            value.0.heap()?.lock().audit_reference_count(value.0.id())?,
            usize::from(u16::MAX) - 1
        );
        drop(slots);
        drop(aliases);
        assert_eq!(
            value.semantic_reference_count()?,
            usize::from(u16::MAX) - usize::from(!saturates)
        );
    }
    Ok(())
}

#[test]
fn invalid_batch_publishes_neither_values_nor_reference_history() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = list(&owner, vec![])?;
    let slots = list(&owner, vec![Sexp::Nil])?;
    assert_eq!(
        slots.replace_shared(&[(0, Sexp::ListObject(value.clone())), (2, Sexp::Nil)]),
        Err(GraphError::IndexOutOfBounds)
    );
    count(&value, 0)?;
    assert_eq!(slots.element(0)?, Sexp::Nil);
    Ok(())
}

#[test]
fn promise_completion_changes_only_the_cache_slot_near_saturation() -> Result<(), GraphError> {
    for cache_is_expression_child in [false, true] {
        let owner = Environment::global();
        let child = list(&owner, vec![])?;
        let other = list(&owner, vec![])?;
        let expression = Sexp::List(vec![
            ListElement {
                name: None,
                value: Sexp::ListObject(child.clone())
            };
            usize::from(u16::MAX) - 1
        ]);
        let promise = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
        count(&child, usize::from(u16::MAX) - 1)?;
        let Force::Evaluate(lease) = promise.begin_force()? else {
            panic!("new promise cached")
        };
        let cache = if cache_is_expression_child {
            &child
        } else {
            &other
        };
        lease.complete(&Sexp::ListObject(cache.clone()))?;
        count(&other, usize::from(!cache_is_expression_child))?;
        assert_eq!(
            child.semantic_reference_count()?,
            usize::from(u16::MAX) - usize::from(!cache_is_expression_child)
        );
        drop(promise);
        count(&other, usize::from(!cache_is_expression_child))?;
        assert_eq!(
            child.semantic_reference_count()?,
            usize::from(u16::MAX) - usize::from(!cache_is_expression_child)
        );
        assert_eq!(
            child.0.heap()?.lock().audit_reference_count(child.0.id())?,
            0
        );
    }
    Ok(())
}
