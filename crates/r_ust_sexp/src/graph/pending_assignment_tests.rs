use super::*;
use crate::{ListElement, Sexp};

fn value(owner: &Environment) -> Result<ListObject, GraphError> {
    ListObject::from_value(
        owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )
}

#[test]
fn pending_move_skips_old_decrement_and_clears_only_on_changed_binding() -> Result<(), GraphError> {
    let owner = Environment::global();
    let list = value(&owner)?;
    owner.insert("x".into(), &Sexp::ListObject(list.clone()))?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    owner.insert("moved".into(), &Sexp::ListObject(list.clone()))?;
    owner.insert("x".into(), &Sexp::Nil)?;
    assert_eq!(list.semantic_reference_count()?, 2);
    assert_eq!(list.0.heap()?.lock().audit_reference_count(list.0.id())?, 1);
    assert!(!owner.finish_assignment("x")?);
    owner.insert("moved".into(), &Sexp::Nil)?;
    assert_eq!(list.semantic_reference_count()?, 1);
    assert_eq!(list.0.heap()?.lock().audit_reference_count(list.0.id())?, 0);
    Ok(())
}

#[test]
fn unchanged_direct_pointer_keeps_pending_and_normal_finish_releases_old() -> Result<(), GraphError>
{
    let owner = Environment::global();
    let list = value(&owner)?;
    owner.insert("x".into(), &Sexp::ListObject(list.clone()))?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    owner.insert("x".into(), &Sexp::ListObject(list.clone()))?;
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    assert_eq!(list.semantic_reference_count()?, 1);
    assert!(owner.finish_assignment("x")?);
    owner.insert("x".into(), &Sexp::Nil)?;
    assert_eq!(list.semantic_reference_count()?, 0);
    assert_eq!(owner.begin_assignment("absent")?, None);
    Ok(())
}

#[test]
fn failed_import_does_not_clear_pending_or_publish_counts() -> Result<(), GraphError> {
    let owner = Environment::global();
    let list = value(&owner)?;
    owner.insert("x".into(), &Sexp::ListObject(list.clone()))?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    let other_heap = Arc::new(Heap::default());
    let other = other_heap.session();
    let foreign = value(&other)?;
    assert_eq!(
        owner.insert("x".into(), &Sexp::ListObject(foreign)),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    assert_eq!(list.semantic_reference_count()?, 1);
    assert_eq!(owner.get_local("x")?, Some(Sexp::ListObject(list.clone())));
    assert!(owner.finish_assignment("x")?);
    Ok(())
}

#[test]
fn pending_metadata_does_not_keep_unreachable_values_alive() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let list = value(&owner)?;
    let id = list.0.id();
    owner.insert("x".into(), &Sexp::ListObject(list.clone()))?;
    owner.begin_assignment("x")?;
    owner.insert("x".into(), &Sexp::Nil)?;
    drop(list);
    assert_eq!(heap.lock().node(id).err(), Some(GraphError::StaleNode));
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    Ok(())
}

#[test]
fn locked_and_empty_frames_cannot_acquire_pending_metadata() -> Result<(), GraphError> {
    let owner = Environment::global();
    let empty = owner.session_root(EnvironmentRole::Empty)?;
    let base = owner.session_root(EnvironmentRole::Base)?;
    assert_eq!(
        empty.begin_assignment("x"),
        Err(GraphError::EmptyEnvironment)
    );
    assert_eq!(
        base.begin_assignment("x"),
        Err(GraphError::EnvironmentLocked)
    );
    assert_eq!(
        empty.finish_assignment("x"),
        Err(GraphError::EmptyEnvironment)
    );
    Ok(())
}
