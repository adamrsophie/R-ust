use super::*;
use crate::{ClosureEnvironment, Environment, EnvironmentRole, Sexp};

fn captured(frame: &Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn environment_rejects_invalid_mutation_protected_roots() {
    let global = Environment::global();
    global
        .install_base(&[("existing".into(), Sexp::Integer(vec![Some(17)]))])
        .unwrap();
    for role in [EnvironmentRole::Base, EnvironmentRole::BaseNamespace] {
        let root = global.session_root(role).unwrap();
        let child = root.child().unwrap();
        assert_eq!(
            root.insert("existing".into(), &Sexp::Nil),
            Err(GraphError::BindingLocked)
        );
        assert_eq!(
            root.insert("fresh".into(), &Sexp::Nil),
            Err(GraphError::EnvironmentLocked)
        );
        assert_eq!(
            child.assign_parent("existing".into(), &Sexp::Nil),
            Err(GraphError::BindingLocked)
        );
        assert_eq!(
            root.get_local("existing").unwrap(),
            Some(Sexp::Integer(vec![Some(17)]))
        );
        assert_eq!(root.local_len().unwrap(), 1);
        assert_eq!(root.get_local("fresh").unwrap(), None);
    }
    let empty = global.session_root(EnvironmentRole::Empty).unwrap();
    assert_eq!(
        empty.insert("fresh".into(), &Sexp::Nil),
        Err(GraphError::EmptyEnvironment)
    );
    assert!(!empty.is_locked().unwrap());
    assert_eq!(empty.local_len().unwrap(), 0);
    assert_eq!(empty.get("existing").unwrap(), None);
    for role in [EnvironmentRole::Global, EnvironmentRole::Autoloads] {
        let root = global.session_root(role).unwrap();
        root.insert("writable".into(), &Sexp::Nil).unwrap();
        assert!(root.contains_local("writable").unwrap());
    }
}

#[test]
fn environment_rejects_invalid_mutation_foreign_heap_is_atomic() {
    let global = Environment::global();
    let child = global.child().unwrap();
    global
        .insert("existing".into(), &Sexp::Integer(vec![Some(23)]))
        .unwrap();
    let heap = Arc::new(Heap::default());
    let foreign = heap.session();
    let value = Sexp::Vector(vec![captured(&child), captured(&foreign)]);
    assert_eq!(
        global.insert("existing".into(), &value),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(
        child.assign_parent("existing".into(), &value),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(
        global.get_local("existing").unwrap(),
        Some(Sexp::Integer(vec![Some(23)]))
    );
    assert_eq!(global.local_len().unwrap(), 1);
    assert_eq!(child.local_len().unwrap(), 0);
    assert!(global.0.heap().unwrap().lock().node(child.0.id()).is_ok());
    assert!(heap.lock().node(foreign.0.id()).is_ok());
}

#[test]
fn environment_rejects_invalid_mutation_bootstrap_is_atomic_and_seals_aliases() {
    let global = Environment::global();
    let base = global.session_root(EnvironmentRole::Base).unwrap();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace).unwrap();
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    let bindings = [
        ("first".into(), captured(&namespace)),
        ("foreign".into(), captured(&foreign)),
    ];
    assert_eq!(global.install_base(&bindings), Err(GraphError::ForeignHeap));
    assert_eq!(base.local_len().unwrap(), 0);
    assert_eq!(namespace.local_len().unwrap(), 0);
    assert_eq!(
        namespace.insert("before_bootstrap".into(), &Sexp::Nil),
        Err(GraphError::EnvironmentLocked)
    );
    global
        .install_base(&[("first".into(), captured(&namespace))])
        .unwrap();
    for root in [base, namespace.clone()] {
        assert_eq!(root.get_local("first").unwrap(), Some(captured(&namespace)));
        assert_eq!(
            root.insert("first".into(), &Sexp::Nil),
            Err(GraphError::BindingLocked)
        );
        assert_eq!(
            root.insert("second".into(), &Sexp::Nil),
            Err(GraphError::EnvironmentLocked)
        );
        assert_eq!(root.local_len().unwrap(), 1);
    }
}
