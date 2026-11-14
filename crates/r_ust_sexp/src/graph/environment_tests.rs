use super::*;
use crate::{Environment, EnvironmentRole, EvaluationOrigin, Force, PromiseState, Sexp};

#[test]
fn environment_topology_parent_chain() {
    let global = Environment::global();
    let autoloads = global.parent().unwrap().expect("global encloses Autoloads");
    let base = autoloads
        .parent()
        .unwrap()
        .expect("Autoloads encloses base");
    let empty = base.parent().unwrap().expect("base encloses empty");
    assert_eq!(empty.parent().unwrap(), None);
    assert_eq!(autoloads.get_local(".Autoloaded").unwrap(), Some(Sexp::Nil));
    assert_eq!(global.get_local(".Autoloaded").unwrap(), None);
    assert_ne!(global, autoloads);
    assert_ne!(base, empty);
}

#[test]
fn environment_topology_final_drop_reclaims_all_roots() {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let global = heap.session();
    let ids = heap
        .lock()
        .frame(global.0.id())
        .unwrap()
        .session
        .unwrap()
        .ids();
    drop(global);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    assert_eq!(
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count(),
        0
    );
    drop(heap);
    assert!(weak.upgrade().is_none());
}

#[test]
fn environment_topology_roles_names_and_shared_store() {
    let global = Environment::global();
    let other = Environment::default();
    let rows = [
        (EnvironmentRole::Global, "R_GlobalEnv", false, 0),
        (EnvironmentRole::Autoloads, "Autoloads", false, 1),
        (EnvironmentRole::Base, "base", true, 0),
        (EnvironmentRole::BaseNamespace, "base", true, 0),
        (EnvironmentRole::Empty, "R_EmptyEnv", false, 0),
    ];
    for (role, name, locked, count) in rows {
        let root = global.session_root(role).unwrap();
        assert_eq!(root.role().unwrap(), role);
        assert_eq!(root.name().unwrap(), name);
        assert_eq!(root.is_locked().unwrap(), locked);
        assert_eq!(root.local_len().unwrap(), count);
        assert_ne!(root, other.session_root(role).unwrap());
        let attribute = if role == EnvironmentRole::Autoloads {
            Some("Autoloads")
        } else {
            None
        };
        assert_eq!(root.name_attribute().unwrap(), attribute);
    }
    let base = global.session_root(EnvironmentRole::Base).unwrap();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace).unwrap();
    assert_ne!(base, namespace);
    assert_eq!(namespace.parent().unwrap(), Some(global.clone()));
    let child = namespace.child().unwrap();
    assert_eq!(child.role().unwrap(), EnvironmentRole::Ordinary);
    assert_eq!(child.name().unwrap(), "");
    assert_eq!(child.name_attribute().unwrap(), None);
    assert_eq!(
        child.session_root(EnvironmentRole::Ordinary),
        Err(GraphError::WrongKind)
    );
    namespace
        .install_base(&[("identity".into(), Sexp::Integer(vec![Some(37)]))])
        .unwrap();
    assert_eq!(
        base.get_local("identity").unwrap(),
        Some(Sexp::Integer(vec![Some(37)]))
    );
    assert_eq!(
        namespace.get_local("identity").unwrap(),
        Some(Sexp::Integer(vec![Some(37)]))
    );
    assert_eq!(base.local_len().unwrap(), 1);
    assert_eq!(namespace.local_len().unwrap(), 1);
    assert!(namespace.contains_local("identity").unwrap());
    base.install_base(&[("replacement".into(), Sexp::Nil)])
        .unwrap();
    assert_eq!(namespace.get_local("replacement").unwrap(), None);
    // A single physical binding store is part of the base/namespace contract.
    let heap = global.0.heap().unwrap();
    let store = heap.lock();
    assert!(std::ptr::eq(
        store.binding_frame(base.0.id()).unwrap(),
        store.binding_frame(namespace.0.id()).unwrap()
    ));
}

#[test]
fn environment_local_count_and_lookup_preserve_pending_promises_and_null() {
    let global = Environment::global();
    let child = global.child().unwrap();
    global
        .insert("inherited".into(), &Sexp::Integer(vec![Some(19)]))
        .unwrap();
    let state = PromiseState::new(
        &Sexp::Symbol("inherited".into()),
        &global,
        EvaluationOrigin::Global,
        false,
    )
    .unwrap();
    child
        .insert(
            "pending".into(),
            &Sexp::Promise {
                state: state.clone(),
            },
        )
        .unwrap();
    child.insert("nil".into(), &Sexp::Nil).unwrap();
    assert_eq!(child.local_len().unwrap(), 2);
    assert_eq!(child.get_local("inherited").unwrap(), None);
    assert_eq!(
        child.get("inherited").unwrap(),
        Some(Sexp::Integer(vec![Some(19)]))
    );
    assert_eq!(child.get_local("nil").unwrap(), Some(Sexp::Nil));
    assert_eq!(child.get_local("absent").unwrap(), None);
    assert_eq!(
        child.get_local("pending").unwrap(),
        Some(Sexp::Promise {
            state: state.clone()
        })
    );
    let Force::Evaluate(lease) = state.begin_force().unwrap() else {
        panic!("metadata forced promise")
    };
    assert!(!lease.interrupted);
    lease.complete(&Sexp::Integer(vec![Some(19)])).unwrap();
}
