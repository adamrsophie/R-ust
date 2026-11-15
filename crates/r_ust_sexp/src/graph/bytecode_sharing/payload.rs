use super::*;

#[test]
fn constant_and_tag_edges_survive_and_collection_preserves_conservative_history(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = heap.session();
    child.insert("marker".into(), &Sexp::Integer(vec![Some(7)]))?;
    let child_id = child.0.id();
    let bytecode = BytecodeObject::new(
        &owner,
        BytecodeParts {
            code: vec![12, 16, 0, 1],
            constants: vec![Sexp::Environment(child.clone())],
            expression: Sexp::Environment(child.clone()),
        },
    )?;
    let code_id = bytecode.0.id();
    assert_eq!(references(&heap, child_id)?, 2);
    let parts = bytecode.parts()?;
    assert_eq!(parts.constants, vec![Sexp::Environment(child.clone())]);
    assert_eq!(parts.expression, Sexp::Environment(child.clone()));
    drop(parts);
    drop(bytecode);
    assert!(matches!(
        heap.lock().node(code_id),
        Err(GraphError::StaleNode)
    ));
    assert_eq!(references(&heap, child_id)?, 2);
    assert_eq!(heap.lock().audit_reference_count(child_id)?, 0);
    assert_eq!(
        child.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(7)]))
    );
    drop(child);
    assert!(matches!(
        heap.lock().node(child_id),
        Err(GraphError::StaleNode)
    ));
    Ok(())
}

#[test]
fn stored_bytecode_identity_preserves_pending_noop_and_changed_write_history(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let first = code(&owner, Sexp::Integer(vec![Some(1)]))?;
    let second = code(&owner, Sexp::Integer(vec![Some(2)]))?;
    owner.insert("x".into(), &Sexp::Bytecode(first.clone()))?;
    assert_eq!(references(&heap, first.0.id())?, 1);
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    let alias = owner.get_local("x")?.ok_or(GraphError::WrongKind)?;
    owner.insert("x".into(), &alias)?;
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    assert_eq!(references(&heap, first.0.id())?, 1);
    owner.insert("x".into(), &Sexp::Bytecode(second.clone()))?;
    assert_eq!(references(&heap, first.0.id())?, 1);
    assert_eq!(references(&heap, second.0.id())?, 1);
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    assert!(owner.finish_assignment("x")?);
    owner.insert("x".into(), &Sexp::Bytecode(first.clone()))?;
    assert_eq!(references(&heap, first.0.id())?, 2);
    assert_eq!(references(&heap, second.0.id())?, 0);
    Ok(())
}

#[test]
fn bytecode_import_and_cycle_collection_follow_managed_handle_liveness() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = heap.session();
    let bytecode = BytecodeObject::new(
        &owner,
        BytecodeParts {
            code: vec![12, 16, 0, 1],
            constants: vec![Sexp::Environment(child.clone())],
            expression: Sexp::Nil,
        },
    )?;
    child.insert("code".into(), &Sexp::Bytecode(bytecode.clone()))?;
    let code_id = bytecode.0.id();
    let child_id = child.0.id();
    owner.insert("code".into(), &Sexp::Bytecode(bytecode.clone()))?;
    drop(bytecode);
    drop(child);
    let Some(Sexp::Bytecode(exported)) = owner.get_local("code")? else {
        panic!("bytecode")
    };
    drop(owner);
    let Sexp::Environment(child) = exported.source()? else {
        panic!("child")
    };
    assert_eq!(
        child.get_local("code")?,
        Some(Sexp::Bytecode(exported.clone()))
    );
    drop(child);
    drop(exported);
    assert!(matches!(
        heap.lock().node(code_id),
        Err(GraphError::StaleNode)
    ));
    assert!(matches!(
        heap.lock().node(child_id),
        Err(GraphError::StaleNode)
    ));
    Ok(())
}
