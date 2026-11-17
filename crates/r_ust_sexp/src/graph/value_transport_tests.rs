use super::*;
use crate::{
    Environment, EvaluationOrigin, Force, ListElement, ListObject, PromiseMetadata, PromiseState,
    Sexp, SexpType, ValueImport, ValueObject,
};

fn managed(value: Sexp) -> ValueObject {
    let Sexp::Managed(value) = value else {
        panic!("managed transport")
    };
    value
}

fn imported(owner: &Environment) -> Result<ValueObject, GraphError> {
    ValueObject::import(
        owner,
        &Sexp::Vector(vec![Sexp::Integer(vec![Some(7)])]),
        ValueImport::Evaluated,
    )
}

#[test]
fn aggregate_transport_reimport_preserves_identity_type_and_mode() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = Sexp::Symbol("code".into());
    let value = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    let transport = value.transport()?;
    assert_eq!(transport.sexptype(), SexpType::Symbol);
    for mode in [
        ValueImport::Code,
        ValueImport::Evaluated,
        ValueImport::Explicit,
    ] {
        let imported = ValueObject::import(&owner, &transport, mode)?;
        assert!(imported.same_object(&value));
        assert_eq!(imported.view()?.mode, Some(ValueImport::Code));
    }
    assert_eq!(value.semantic_reference_count()?, 0);
    assert_eq!(transport.snapshot()?, raw);
    Ok(())
}

#[test]
fn aggregate_transport_bindings_own_outer_node_without_reowning_children() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = imported(&owner)?;
    let child = value.view()?.children[0].clone();
    let transport = value.transport()?;
    owner.insert("a".into(), &transport)?;
    owner.insert("b".into(), &transport)?;
    assert_eq!(value.semantic_reference_count()?, 2);
    assert_eq!(child.semantic_reference_count()?, 1);
    let a = managed(owner.get_local("a")?.unwrap());
    let b = managed(owner.get("b")?.unwrap());
    assert!(a.same_object(&b));
    assert!(a.same_object(&value));
    assert_eq!(
        owner.get_snapshot("a")?,
        Some(Sexp::Vector(vec![Sexp::Integer(vec![Some(7)])]))
    );
    assert_eq!(owner.get_local_snapshot("absent")?, None);
    assert_eq!(value.semantic_reference_count()?, 2);
    owner.insert("a".into(), &Sexp::Nil)?;
    owner.insert("b".into(), &Sexp::Nil)?;
    assert_eq!(value.semantic_reference_count()?, 0);
    assert_eq!(child.semantic_reference_count()?, 1);
    Ok(())
}

#[test]
fn aggregate_transport_same_pending_rebind_and_external_leases_keep_history(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = imported(&owner)?;
    let transport = value.transport()?;
    owner.insert("x".into(), &transport)?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    owner.insert("x".into(), &transport)?;
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    assert_eq!(value.semantic_reference_count()?, 1);
    let lease = owner.retain_value(&transport)?;
    let clone = lease.clone();
    let second = value.retain()?;
    assert_eq!(value.semantic_reference_count()?, 3);
    drop(lease);
    assert_eq!(value.semantic_reference_count()?, 3);
    drop(clone);
    drop(second);
    assert_eq!(value.semantic_reference_count()?, 1);
    owner.insert("x".into(), &Sexp::Nil)?;
    // Pending assignment intentionally suppresses the old decrement.
    assert_eq!(value.semantic_reference_count()?, 1);
    assert_eq!(heap.lock().audit_reference_count(value.0.id())?, 0);
    Ok(())
}

#[test]
fn aggregate_transport_promises_keep_expression_and_value_ownership_distinct(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let code = ValueObject::import(&owner, &Sexp::Symbol("x".into()), ValueImport::Code)?;
    let value = imported(&owner)?;
    let expression = code.transport()?;
    let ordinary = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
    let scratch = PromiseState::new_replacement(
        &expression,
        &owner,
        PromiseMetadata {
            origin: EvaluationOrigin::Global,
            supplied: true,
        },
    )?;
    assert_eq!(code.semantic_reference_count()?, 2);
    assert!(managed(ordinary.expression()?).same_object(&code));
    for promise in [&ordinary, &scratch] {
        let Force::Evaluate(lease) = promise.begin_force()? else {
            panic!("unforced")
        };
        assert!(managed(lease.expression.clone()).same_object(&code));
        lease.complete(&value.transport()?)?;
        let Force::Cached(cached) = promise.begin_force()? else {
            panic!("cached")
        };
        assert!(managed(cached).same_object(&value));
    }
    assert_eq!(value.semantic_reference_count()?, 1);
    assert_eq!(value.view()?.children[0].semantic_reference_count()?, 1);
    Ok(())
}

#[test]
fn aggregate_transport_explicit_list_slots_preserve_managed_aliases() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = imported(&owner)?;
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: Some("item".into()),
            value: value.transport()?,
        }]),
    )?;
    assert_eq!(value.semantic_reference_count()?, 1);
    assert!(managed(list.element(0)?).same_object(&value));
    let explicit = ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?;
    let Sexp::ListObject(exported) = explicit.transport()? else {
        panic!("explicit presentation")
    };
    assert!(exported.same_object(&list));
    list.replace_shared(&[(0, Sexp::Nil)])?;
    assert_eq!(value.semantic_reference_count()?, 0);
    Ok(())
}

#[test]
fn aggregate_transport_managed_and_explicit_cycle_reclaims_on_final_release(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let count = || {
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    };
    let before = count();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let value = ValueObject::import(
        &owner,
        &Sexp::Vector(vec![Sexp::ListObject(list.clone())]),
        ValueImport::Evaluated,
    )?;
    list.replace_shared(&[(0, value.transport()?)])?;
    assert_eq!(value.semantic_reference_count()?, 1);
    let snapshot = value.snapshot()?;
    assert!(
        matches!(snapshot, Sexp::Vector(ref values) if matches!(values[0], Sexp::ListObject(_)))
    );
    drop(snapshot);
    drop(list);
    drop(value);
    assert_eq!(count(), before);
    Ok(())
}

#[test]
fn aggregate_transport_explicit_presentation_alias_is_the_same_pending_reference(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let list = ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    let explicit = Sexp::ListObject(list.clone());
    let handle = ValueObject::import(&owner, &explicit, ValueImport::Explicit)?;
    owner.insert("x".into(), &explicit)?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    owner.insert("x".into(), &Sexp::Managed(handle))?;
    assert_eq!(list.semantic_reference_count()?, 1);
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    owner.finish_assignment("x")?;
    owner.insert("x".into(), &Sexp::Nil)?;
    assert_eq!(list.semantic_reference_count()?, 0);
    Ok(())
}
