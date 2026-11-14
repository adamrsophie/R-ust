use super::*;
use crate::{
    canonical_language, Environment, LanguageError, ListElement, ListObject, PairListObject, Sexp,
    SexpType,
};

fn slot(value: Sexp) -> ListElement {
    ListElement { name: None, value }
}

fn vector(owner: &Environment) -> Result<ListObject, GraphError> {
    ListObject::from_value(owner, &Sexp::List(vec![slot(Sexp::Nil), slot(Sexp::Nil)]))
}

#[test]
fn shared_values_vector_alias_and_copy() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = vector(&owner)?;
    value.set_gp_shared(1)?;
    owner.insert("value".into(), &Sexp::ListObject(value.clone()))?;
    let Some(Sexp::ListObject(alias)) = owner.get_local("value")? else {
        panic!("list export")
    };
    assert!(value.same_object(&alias));
    value.replace_shared(&[(0, Sexp::Char(Some("error".into())))])?;
    assert_eq!(alias.element(0)?, Sexp::Char(Some("error".into())));
    let copied = value.copy_with(&value.value()?)?;
    assert!(!copied.same_object(&value));
    assert_eq!(copied.header()?.gp(), 1);
    copied.replace_shared(&[(0, Sexp::Nil)])?;
    assert_ne!(copied.element(0)?, value.element(0)?);
    assert_eq!(Sexp::ListObject(value).sexptype(), SexpType::Vector);
    Ok(())
}

#[test]
fn shared_values_atomic_rejection() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = vector(&owner)?;
    value.set_gp_shared(7)?;
    let before = value.value()?;
    assert_eq!(
        value.replace_shared(&[(0, Sexp::Integer(vec![Some(1)])), (2, Sexp::Nil)]),
        Err(GraphError::IndexOutOfBounds)
    );
    let foreign_heap = Arc::new(Heap::default());
    let foreign = super::super::Environment(foreign_heap.insert(Node::Frame(Frame::default())));
    assert_eq!(
        value.replace_shared(&[(0, Sexp::Nil), (1, Sexp::Environment(foreign))]),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(value.value()?, before);
    assert_eq!(value.header()?.gp(), 7);
    assert_eq!(
        value.replace_shared(&[(0, Sexp::Char(Some("a\0b".into())))]),
        Err(GraphError::InvalidCharacter)
    );
    assert_eq!(value.value()?, before);
    assert_eq!(value.element(2), Err(GraphError::IndexOutOfBounds));
    assert_eq!(value.copy_with(&Sexp::Nil), Err(GraphError::WrongKind));
    let unrooted = super::super::ListObject(Link::Edge(value.0.id()));
    assert_eq!(unrooted.value(), Err(GraphError::Unrooted));
    value.replace_shared(&[(0, Sexp::Integer(vec![Some(1)])), (0, Sexp::Nil)])?;
    assert_eq!(value.value()?, before);
    value.replace_shared(&[])?;
    Ok(())
}

#[test]
fn shared_values_cons_nonancestor() -> Result<(), GraphError> {
    let owner = Environment::global();
    let entry = vector(&owner)?;
    let tail = PairListObject::cons(&owner, &slot(Sexp::ListObject(entry.clone())), None)?;
    let a = PairListObject::cons(&owner, &slot(Sexp::Nil), Some(&tail))?;
    let b = PairListObject::cons(&owner, &slot(Sexp::Nil), Some(&tail))?;
    assert!(!a.same_object(&b));
    for head in [&a, &b, &a] {
        owner.insert("head".into(), &Sexp::PairListObject(head.clone()))?;
        let Some(Sexp::PairListObject(exported)) = owner.get_local("head")? else {
            panic!("cons export")
        };
        assert!(head.same_object(&exported));
        let Some(exported_tail) = exported.cdr()? else {
            panic!("tail")
        };
        assert!(tail.same_object(&exported_tail));
        let Sexp::ListObject(exported_entry) = exported_tail.car()?.value else {
            panic!("entry")
        };
        assert!(entry.same_object(&exported_entry));
    }
    assert_eq!(a.elements()?.len(), 2);
    assert_eq!(Sexp::PairListObject(a.clone()).sexptype(), SexpType::List);
    let heap = Arc::clone(owner.0.heap()?);
    let tail_id = tail.0.id();
    let entry_id = entry.0.id();
    drop(tail);
    drop(entry);
    drop(owner);
    for id in [tail_id, entry_id] {
        assert!(
            heap.lock().node(id).is_ok(),
            "both heads retain shared tail and entry"
        );
    }
    drop(a);
    {
        let Some(retained_tail) = b.cdr()? else {
            panic!("remaining head retains tail")
        };
        assert_eq!(retained_tail.0.id(), tail_id);
        let Sexp::ListObject(retained_entry) = retained_tail.car()?.value else {
            panic!("remaining tail retains entry")
        };
        assert_eq!(retained_entry.0.id(), entry_id);
    }
    for id in [tail_id, entry_id] {
        assert!(
            heap.lock().node(id).is_ok(),
            "remaining head retains shared tail and entry"
        );
    }
    drop(b);
    for id in [tail_id, entry_id] {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    Ok(())
}

#[test]
fn shared_values_cons_lifetime() -> Result<(), GraphError> {
    let owner = Environment::global();
    let heap = Arc::clone(owner.0.heap()?);
    let value = vector(&owner)?;
    let head = PairListObject::cons(&owner, &slot(Sexp::ListObject(value.clone())), None)?;
    value.replace_shared(&[
        (0, Sexp::Environment(owner.clone())),
        (1, Sexp::PairListObject(head.clone())),
    ])?;
    owner.insert("head".into(), &Sexp::PairListObject(head.clone()))?;
    let ids = [owner.0.id(), value.0.id(), head.0.id()];
    let alias = head.clone();
    drop(head);
    drop(value);
    drop(owner);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    drop(alias);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    Ok(())
}

#[test]
fn shared_values_boundaries() -> Result<(), GraphError> {
    let owner = Environment::global();
    let value = vector(&owner)?;
    let cons = PairListObject::cons(&owner, &slot(Sexp::Nil), None)?;
    for value in [
        Sexp::ListObject(value),
        Sexp::PairListObject(cons),
        Sexp::Char(None),
    ] {
        for candidate in [
            value.clone(),
            Sexp::Call {
                function: Box::new(Sexp::Symbol("identity".into())),
                arguments: vec![crate::CallArgument::positional(value)],
            },
        ] {
            let Err(error) = canonical_language(&candidate) else {
                panic!("runtime value accepted as parser-origin language")
            };
            assert_eq!(error, LanguageError::RuntimeValue);
            assert_eq!(
                error.to_string(),
                "runtime value is not parser-origin language"
            );
        }
    }
    assert_eq!(Sexp::List(vec![]).sexptype(), SexpType::Vector);
    assert_eq!(Sexp::Char(None).sexptype(), SexpType::Char);
    Ok(())
}
