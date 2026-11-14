use super::*;
use crate::{ClosureEnvironment, EnvironmentRole, Sexp};

fn captured(frame: &crate::Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn retained_child_origin_global_is_traced() {
    for role in [EnvironmentRole::Empty, EnvironmentRole::Base] {
        let heap = Arc::new(Heap::default());
        let weak = Arc::downgrade(&heap);
        let global = heap.session();
        let global_id = global.0.id();
        let ids = heap.lock().frame(global_id).unwrap().session.unwrap().ids();
        let parent = global.session_root(role).unwrap();
        let child = parent.child().unwrap();
        let child_id = child.0.id();
        drop(parent);
        drop(global);
        assert!(
            heap.lock().node(global_id).is_ok(),
            "{role:?} lost origin global"
        );
        let consumer = heap.session();
        consumer
            .insert("retained".into(), &captured(&child))
            .unwrap();
        for value in [41, 42] {
            child
                .assign_parent("origin_only".into(), &Sexp::Integer(vec![Some(value)]))
                .unwrap();
            let origin = child.session_root(EnvironmentRole::Global).unwrap();
            assert_eq!(
                origin.get_local("origin_only").unwrap(),
                Some(Sexp::Integer(vec![Some(value)]))
            );
            assert_eq!(consumer.get_local("origin_only").unwrap(), None);
            assert_eq!(child.get_local("origin_only").unwrap(), None);
            assert_eq!(
                child
                    .parent()
                    .unwrap()
                    .unwrap()
                    .get_local("origin_only")
                    .unwrap(),
                None
            );
        }
        drop(consumer);
        drop(child);
        for id in ids.into_iter().chain([child_id]) {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        drop(heap);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn retained_base_namespace_final_root_collection() {
    for retained_role in [EnvironmentRole::Base, EnvironmentRole::BaseNamespace] {
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
        let base = global.session_root(EnvironmentRole::Base).unwrap();
        let namespace = global.session_root(EnvironmentRole::BaseNamespace).unwrap();
        let bindings = [("identity".into(), captured(&namespace))];
        global.install_base(&bindings).unwrap();
        let retained = global.session_root(retained_role).unwrap();
        drop(bindings);
        drop(base);
        drop(namespace);
        drop(global);
        for id in ids {
            assert!(heap.lock().node(id).is_ok());
        }
        let exported = retained.get_local("identity").unwrap().unwrap();
        assert_eq!(retained.local_len().unwrap(), 1);
        assert_eq!(
            retained.insert("identity".into(), &Sexp::Nil),
            Err(GraphError::BindingLocked)
        );
        assert_eq!(
            retained.insert("new".into(), &Sexp::Nil),
            Err(GraphError::EnvironmentLocked)
        );
        drop(retained);
        for id in ids {
            assert!(heap.lock().node(id).is_ok());
        }
        drop(exported);
        for id in ids {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        let replacement = heap.session();
        for id in ids {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        drop(replacement);
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
}
