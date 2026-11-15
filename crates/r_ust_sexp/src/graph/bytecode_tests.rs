use super::*;
use crate::{BytecodeObject, BytecodeParts, Environment, Sexp, SexpType};

fn parts(constants: Vec<Sexp>, expression: Sexp) -> BytecodeParts {
    BytecodeParts {
        code: vec![12, 16, 0, 1],
        constants,
        expression,
    }
}

#[test]
fn bytecode_identity_source_metadata_and_actual_address() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let input = parts(
        vec![Sexp::Integer(vec![Some(42)])],
        Sexp::Symbol("metadata".into()),
    );
    let code = BytecodeObject::new(&owner, input.clone())?;
    let alias = code.clone();
    let other = BytecodeObject::new(&owner, input.clone())?;
    assert_eq!(Sexp::Bytecode(code.clone()).sexptype(), SexpType::Bytecode);
    assert!(code.same_object(&alias));
    assert!(!code.same_object(&other));
    assert_eq!(code.parts()?, input);
    assert_eq!(code.source()?, Sexp::Integer(vec![Some(42)]));
    assert_eq!(code.display_address()?, alias.display_address()?);
    assert_ne!(code.display_address()?, other.display_address()?);
    let empty = BytecodeObject::new(&owner, parts(vec![], Sexp::Integer(vec![Some(9)])))?;
    assert_eq!(empty.source()?, Sexp::Nil);
    assert_eq!(empty.parts()?.expression, Sexp::Integer(vec![Some(9)]));
    Ok(())
}

#[test]
fn bytecode_constants_and_metadata_keep_cycles_live_then_release() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let holder = heap.session();
    let constants_frame = heap.session();
    let metadata_frame = heap.session();
    constants_frame.insert("marker".into(), &Sexp::Integer(vec![Some(7)]))?;
    metadata_frame.insert("marker".into(), &Sexp::Integer(vec![Some(8)]))?;
    let code = BytecodeObject::new(
        &holder,
        parts(
            vec![Sexp::Environment(constants_frame.clone())],
            Sexp::Environment(metadata_frame.clone()),
        ),
    )?;
    let ids = [code.0.id(), constants_frame.0.id(), metadata_frame.0.id()];
    constants_frame.insert("cycle".into(), &Sexp::Bytecode(code.clone()))?;
    metadata_frame.insert("cycle".into(), &Sexp::Bytecode(code.clone()))?;
    holder.insert("code".into(), &Sexp::Bytecode(code.clone()))?;
    drop(code);
    drop(constants_frame);
    drop(metadata_frame);
    let Some(Sexp::Bytecode(exported)) = holder.get_local("code")? else {
        panic!("bytecode")
    };
    drop(holder);
    let snapshot = exported.parts()?;
    let Sexp::Environment(constants) = &snapshot.constants[0] else {
        panic!("constant")
    };
    let Sexp::Environment(metadata) = &snapshot.expression else {
        panic!("metadata")
    };
    assert_eq!(
        constants.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(7)]))
    );
    assert_eq!(
        metadata.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(8)]))
    );
    drop(snapshot);
    let address = exported.display_address()?;
    for _ in 0..128 {
        drop(heap.session());
    }
    assert_eq!(exported.display_address()?, address);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    Ok(())
}

#[test]
fn bytecode_constructor_retains_moved_constants_and_rejects_foreign_graphs(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let frame = heap.session();
    let id = frame.0.id();
    let code = BytecodeObject::new(&owner, parts(vec![Sexp::Environment(frame)], Sexp::Nil))?;
    assert!(heap.lock().node(id).is_ok());
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    let foreign_id = foreign.0.id();
    let result = BytecodeObject::new(
        &owner,
        parts(
            vec![Sexp::Bytecode(code.clone()), Sexp::Environment(foreign)],
            Sexp::Nil,
        ),
    );
    assert_eq!(result, Err(GraphError::ForeignHeap));
    assert!(matches!(
        foreign_heap.lock().node(foreign_id),
        Err(GraphError::StaleNode)
    ));
    assert!(heap.lock().node(id).is_ok());
    drop(code);
    assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    Ok(())
}

#[test]
fn nested_bytecode_exports_retain_inner_identity() -> Result<(), GraphError> {
    let owner = Environment::global();
    let inner = BytecodeObject::new(&owner, parts(vec![Sexp::Symbol("x".into())], Sexp::Nil))?;
    let outer = BytecodeObject::new(
        &owner,
        parts(vec![Sexp::Bytecode(inner.clone())], Sexp::Nil),
    )?;
    let Sexp::Bytecode(exported) = outer.source()? else {
        panic!("inner code")
    };
    assert!(inner.same_object(&exported));
    assert_eq!(exported.source()?, Sexp::Symbol("x".into()));
    Ok(())
}
