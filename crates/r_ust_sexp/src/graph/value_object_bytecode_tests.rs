use super::*;
use crate::{BytecodeObject, BytecodeParts, Sexp, SexpType, ValueImport, ValueKind, ValueObject};

fn code(owner: &crate::Environment) -> Result<BytecodeObject, GraphError> {
    BytecodeObject::new(
        owner,
        BytecodeParts {
            code: vec![12, 1],
            constants: vec![Sexp::Integer(vec![Some(7)])],
            expression: Sexp::Integer(vec![Some(7)]),
        },
    )
}

fn allocation_state(heap: &Heap) -> (usize, usize) {
    let store = heap.lock();
    (
        store.slots.len(),
        store
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count(),
    )
}

#[test]
fn aggregate_bytecode_import_is_opaque_and_payload_ownership_stays_single() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let code = code(&owner)?;
    let raw = Sexp::Bytecode(code.clone());
    let before = allocation_state(&heap);
    let first = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    let second = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    assert_eq!(allocation_state(&heap), before);
    assert!(first.same_object(&second));
    let view = first.view()?;
    assert_eq!(view.header.sexptype(), SexpType::Bytecode);
    assert_eq!(view.kind, ValueKind::Explicit);
    assert_eq!(view.mode, None);
    assert!(view.children.is_empty());
    let Sexp::Bytecode(snapshot) = first.snapshot()? else {
        panic!("bytecode handle");
    };
    assert!(code.same_object(&snapshot));

    let holder = ValueObject::import(&owner, &Sexp::Vector(vec![raw]), ValueImport::Evaluated)?;
    assert_eq!(first.semantic_reference_count()?, 1);
    assert!(holder.view()?.children[0].same_object(&first));
    let one = holder.retain()?;
    let two = holder.retain()?;
    assert_eq!(holder.semantic_reference_count()?, 2);
    assert_eq!(first.semantic_reference_count()?, 1);
    drop(code);
    drop(snapshot);
    drop(second);
    drop(first);
    let Sexp::Vector(values) = holder.snapshot()? else {
        panic!("vector");
    };
    let Sexp::Bytecode(snapshot) = &values[0] else {
        panic!("retained bytecode");
    };
    assert_eq!(snapshot.source()?, Sexp::Integer(vec![Some(7)]));
    drop(one);
    drop(two);
    Ok(())
}

#[test]
fn aggregate_bytecode_rejects_foreign_and_wrong_kind_without_publishing() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign = Arc::new(Heap::default()).session();
    for (value, expected) in [
        (Sexp::Bytecode(code(&foreign)?), GraphError::ForeignHeap),
        (
            Sexp::Bytecode(crate::graph::BytecodeObject(owner.0.clone())),
            GraphError::WrongKind,
        ),
    ] {
        let before = allocation_state(&heap);
        let value = Sexp::Vector(vec![Sexp::Integer(vec![Some(1)]), value]);
        assert_eq!(
            ValueObject::import(&owner, &value, ValueImport::Code),
            Err(expected)
        );
        assert_eq!(allocation_state(&heap), before);
    }
    Ok(())
}
