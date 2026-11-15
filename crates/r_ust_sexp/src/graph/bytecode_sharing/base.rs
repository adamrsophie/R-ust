use super::*;

#[test]
fn finalization_acquires_bytecode_binding_and_preserves_default_owners_once(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let global = heap.session();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
    let helper = heap.session();
    let mut original = closure(&namespace, Sexp::Integer(vec![Some(1)]));
    let Sexp::Closure {
        parameters,
        attributes,
        ..
    } = &mut original
    else {
        panic!("closure")
    };
    parameters.push(FunctionParameter {
        name: "x".into(),
        default: Some(Sexp::Environment(helper.clone())),
    });
    attributes.push(crate::ListElement {
        name: Some("owner".into()),
        value: Sexp::Environment(helper.clone()),
    });
    global.install_base(&[("f".into(), original.clone())])?;
    assert_eq!(references(&heap, helper.0.id())?, 2);
    let namespace_before = references(&heap, namespace.0.id())?;
    let (replacement, bytecode) = compiled(&namespace, &original)?;
    assert_eq!(references(&heap, bytecode.0.id())?, 0);
    let replacements = vec![("f".into(), original, replacement.clone())];
    global.finalize_base_bytecode(&replacements)?;
    assert!(global.base_bytecode_ready()?);
    assert_eq!(references(&heap, bytecode.0.id())?, 1);
    assert_eq!(references(&heap, helper.0.id())?, 2);
    assert_eq!(references(&heap, namespace.0.id())?, namespace_before);
    assert_eq!(namespace.get_local("f")?, Some(replacement));
    global.finalize_base_bytecode(&replacements)?;
    assert_eq!(references(&heap, bytecode.0.id())?, 1);
    assert_eq!(references(&heap, helper.0.id())?, 2);
    assert_eq!(references(&heap, namespace.0.id())?, namespace_before);
    Ok(())
}

#[test]
fn invalid_later_binding_and_duplicate_name_leave_batch_and_counts_unchanged(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let global = heap.session();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
    let original = closure(&namespace, Sexp::Integer(vec![Some(1)]));
    global.install_base(&[("f".into(), original.clone())])?;
    let (replacement, bytecode) = compiled(&namespace, &original)?;
    let first = ("f".into(), original.clone(), replacement.clone());
    let namespace_before = references(&heap, namespace.0.id())?;
    let invalid = ("absent".into(), original.clone(), replacement.clone());
    assert_eq!(
        global.finalize_base_bytecode(&[first.clone(), invalid]),
        Err(GraphError::WrongKind)
    );
    assert_eq!(
        global.finalize_base_bytecode(&[first.clone(), first]),
        Err(GraphError::InvalidBytecode)
    );
    assert!(!global.base_bytecode_ready()?);
    assert_eq!(namespace.get_local("f")?, Some(original));
    assert_eq!(namespace.get_local("absent")?, None);
    assert_eq!(references(&heap, bytecode.0.id())?, 0);
    assert_eq!(references(&heap, namespace.0.id())?, namespace_before);
    Ok(())
}

#[test]
fn unexpected_pending_base_binding_rejects_before_publication() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let global = heap.session();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
    let base = global.session_root(EnvironmentRole::Base)?;
    let original = closure(&namespace, Sexp::Integer(vec![Some(1)]));
    global.install_base(&[("f".into(), original.clone())])?;
    let (replacement, bytecode) = compiled(&namespace, &original)?;
    heap.lock()
        .frame_mut(base.0.id())?
        .assignment_pending
        .insert("f".into());
    assert_eq!(
        global.finalize_base_bytecode(&[("f".into(), original.clone(), replacement)]),
        Err(GraphError::WrongKind)
    );
    assert_eq!(namespace.get_local("f")?, Some(original));
    assert!(!global.base_bytecode_ready()?);
    assert_eq!(references(&heap, bytecode.0.id())?, 0);
    assert!(heap
        .lock()
        .frame(base.0.id())?
        .assignment_pending
        .contains("f"));
    Ok(())
}

#[test]
fn foreign_bytecode_in_later_replacement_never_publishes_valid_prefix() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let global = heap.session();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
    let original = closure(&namespace, Sexp::Integer(vec![Some(1)]));
    global.install_base(&[
        ("f".into(), original.clone()),
        ("g".into(), original.clone()),
    ])?;
    let (replacement, bytecode) = compiled(&namespace, &original)?;
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    let foreign_code = code(&foreign, Sexp::Integer(vec![Some(1)]))?;
    let mut invalid = original.clone();
    let Sexp::Closure { body, .. } = &mut invalid else {
        panic!("closure")
    };
    *body = Box::new(Sexp::Bytecode(foreign_code));
    let updates = [
        ("f".into(), original.clone(), replacement),
        ("g".into(), original.clone(), invalid),
    ];
    assert_eq!(
        global.finalize_base_bytecode(&updates),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(namespace.get_local("f")?, Some(original.clone()));
    assert_eq!(namespace.get_local("g")?, Some(original));
    assert!(!global.base_bytecode_ready()?);
    assert_eq!(references(&heap, bytecode.0.id())?, 0);
    Ok(())
}
