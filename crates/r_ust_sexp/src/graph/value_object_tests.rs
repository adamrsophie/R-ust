use super::*;
use crate::{
    ClosureEnvironment, FunctionParameter, ListElement, Sexp, SexpType, ValueImport, ValueKind,
    ValueObject,
};

fn live(heap: &Heap) -> usize {
    heap.lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count()
}

fn closure(owner: &crate::Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![FunctionParameter::optional(
            "x",
            Sexp::Integer(vec![Some(7)]),
        )],
        body: Box::new(Sexp::Symbol("x".into())),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![ListElement {
            name: Some("meta".into()),
            value: Sexp::List(vec![]),
        }],
    }
}

#[test]
fn aggregate_closure_alias_leases_own_outer_node_once_per_lease() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = closure(&owner);
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let view = value.view()?;
    assert_eq!(view.kind, ValueKind::Closure);
    assert_eq!(view.children.len(), 4);
    assert_eq!(view.children[0].view()?.mode, Some(ValueImport::Code));
    assert_eq!(view.children[1].view()?.mode, Some(ValueImport::Code));
    assert_eq!(view.children[3].view()?.mode, Some(ValueImport::Evaluated));
    assert_eq!(view.children[0].view()?.kind, ValueKind::Formals);
    assert_eq!(view.children[3].view()?.kind, ValueKind::Attributes);
    for index in [0, 1, 3] {
        assert_eq!(view.children[index].semantic_reference_count()?, 1);
    }
    assert_eq!(value.semantic_reference_count()?, 0);
    let first = value.retain()?;
    let first_clone = first.clone();
    let second = value.retain()?;
    assert_eq!(value.semantic_reference_count()?, 2);
    for index in [0, 1, 3] {
        assert_eq!(view.children[index].semantic_reference_count()?, 1);
    }
    assert_eq!(value.snapshot()?, raw);
    let again = value.view()?;
    for (left, right) in view.children.iter().zip(&again.children) {
        assert!(left.same_object(right));
    }
    drop(first);
    assert_eq!(value.semantic_reference_count()?, 2);
    drop(first_clone);
    assert_eq!(value.semantic_reference_count()?, 1);
    drop(second);
    assert_eq!(value.semantic_reference_count()?, 0);
    Ok(())
}

#[test]
fn aggregate_equal_imports_are_distinct_and_clones_share_identity() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = closure(&owner);
    let first = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let second = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    assert!(!first.same_object(&second));
    assert!(first.same_object(&first.clone()));
    assert!(!first.view()?.children[1].same_object(&second.view()?.children[1]));
    assert_eq!(first.snapshot()?, second.snapshot()?);
    Ok(())
}

#[test]
fn aggregate_snapshots_cover_reference_containers_and_matrix_storage() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let item = ListElement {
        name: Some("x".into()),
        value: Sexp::Integer(vec![Some(1)]),
    };
    let values = vec![
        Sexp::Expression {
            elements: vec![item.clone()],
            names_present: true,
        },
        Sexp::AttributedCharacter {
            values: vec![Some("text".into())],
            attributes: vec![item.clone()],
        },
        Sexp::DataFrame {
            columns: vec![item.clone()],
            nrow: 1,
        },
        Sexp::Matrix {
            data: Box::new(Sexp::String(vec![Some("text".into())])),
            nrow: 1,
            ncol: 1,
        },
        Sexp::Vector(vec![closure(&owner)]),
        Sexp::List(vec![item.clone()]),
        Sexp::pairlist(vec![item.clone()]),
        Sexp::Dots(vec![item]),
    ];
    for raw in values {
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let first = value.view()?;
        let second = value.view()?;
        for (a, b) in first.children.iter().zip(&second.children) {
            assert!(a.same_object(b));
        }
        if matches!(raw, Sexp::Matrix { .. }) {
            assert_eq!(first.header.sexptype(), SexpType::String);
        }
        assert_eq!(value.snapshot()?, raw);
    }
    Ok(())
}

#[test]
fn aggregate_explicit_handles_preserve_identity_without_wrapper_nodes() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let explicit = crate::ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    let raw = Sexp::ListObject(explicit.clone());
    let before = live(&heap);
    let first = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let second = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    assert_eq!(live(&heap), before);
    assert!(first.same_object(&second));
    assert_eq!(first.view()?.kind, ValueKind::Explicit);
    let Sexp::ListObject(snapshot) = first.snapshot()? else {
        panic!("explicit list")
    };
    assert!(explicit.same_object(&snapshot));
    let holder = ValueObject::import(&owner, &Sexp::Vector(vec![raw]), ValueImport::Evaluated)?;
    assert_eq!(explicit.semantic_reference_count()?, 1);
    let _one = holder.retain()?;
    let _two = holder.retain()?;
    assert_eq!(explicit.semantic_reference_count()?, 1);
    Ok(())
}

#[test]
fn aggregate_code_import_is_inert_and_retains_persistent_child_identity() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = Sexp::Assignment {
        operator: crate::AssignmentOperator::Left,
        target: "never_run".into(),
        value: Box::new(Sexp::Integer(vec![Some(9)])),
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    assert_eq!(value.view()?.kind, ValueKind::Code);
    assert!(value.view()?.children[0].same_object(&value.view()?.children[0]));
    assert_eq!(owner.get_local("never_run")?, None);
    assert_eq!(value.snapshot()?, raw);
    assert_eq!(owner.get_local("never_run")?, None);
    Ok(())
}

#[test]
fn aggregate_scratch_views_protect_children_without_semantic_ownership() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let before = live(&heap);
    let value = ValueObject::import(&owner, &closure(&owner), ValueImport::Evaluated)?;
    let body = value.view()?.children[1].clone();
    assert_eq!(body.semantic_reference_count()?, 1);
    drop(value);
    assert_eq!(body.snapshot()?, Sexp::Symbol("x".into()));
    drop(body);
    assert_eq!(live(&heap), before);
    Ok(())
}

#[test]
fn aggregate_release_reclaims_all_imported_nodes() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let before = live(&heap);
    let value = ValueObject::import(&owner, &closure(&owner), ValueImport::Evaluated)?;
    let lease = value.retain()?;
    drop(value);
    assert!(live(&heap) > before);
    drop(lease);
    assert_eq!(live(&heap), before);
    Ok(())
}

#[test]
fn aggregate_explicit_cycle_stays_opaque_and_reclaims_on_final_release() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let before = live(&heap);
    let list = crate::ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    list.replace_shared(&[(0, Sexp::ListObject(list.clone()))])?;
    let value = ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Evaluated,
    )?;
    let Sexp::ListObject(snapshot) = value.snapshot()? else {
        panic!("explicit list")
    };
    assert!(snapshot.same_object(&list));
    drop(snapshot);
    drop(list);
    drop(value);
    assert_eq!(live(&heap), before);
    Ok(())
}
