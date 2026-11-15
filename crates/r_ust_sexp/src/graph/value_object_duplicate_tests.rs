use super::*;
use crate::{ClosureEnvironment, FunctionParameter, ListElement, Sexp, ValueImport, ValueObject};

#[test]
fn aggregate_closure_shallow_copy_shares_code_and_copies_attribute_container(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = Sexp::Closure {
        parameters: vec![FunctionParameter::required("x")],
        body: Box::new(Sexp::Symbol("x".into())),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![ListElement {
            name: Some("custom".into()),
            value: Sexp::List(vec![]),
        }],
    };
    let original = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let copy = original.shallow_copy_closure()?;
    assert!(!original.same_object(&copy));
    assert_eq!(copy.snapshot()?, raw);
    let before = original.view()?;
    let after = copy.view()?;
    for i in 0..3 {
        assert!(before.children[i].same_object(&after.children[i]));
    }
    assert!(!before.children[3].same_object(&after.children[3]));
    let attr_before = before.children[3].view()?;
    let attr_after = after.children[3].view()?;
    assert!(attr_before.children[0].same_object(&attr_after.children[0]));
    assert_eq!(attr_before.children[0].semantic_reference_count()?, 2);
    for i in [0, 1] {
        assert_eq!(before.children[i].semantic_reference_count()?, 2);
    }
    assert_eq!(before.children[3].semantic_reference_count()?, 1);
    assert_eq!(after.children[3].semantic_reference_count()?, 1);
    Ok(())
}

#[test]
fn aggregate_nonclosure_copy_rejects_without_publication() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(&owner, &Sexp::List(vec![]), ValueImport::Evaluated)?;
    let count = heap.lock().slots.len();
    assert_eq!(
        value.shallow_copy_closure().unwrap_err(),
        GraphError::WrongKind
    );
    assert_eq!(heap.lock().slots.len(), count);
    Ok(())
}
