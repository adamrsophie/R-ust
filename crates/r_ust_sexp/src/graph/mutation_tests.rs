use super::*;
use crate::{ClosureEnvironment, Environment, Sexp};

#[test]
fn removing_last_stored_edge_collects_without_another_root_drop() {
    for superassignment in [false, true] {
        let parent = Environment::global();
        let caller = parent.child().unwrap();
        let child = parent.child().unwrap();
        let heap = Arc::clone(parent.0.heap().unwrap());
        let child_id = child.0.id();
        let value = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Nil),
            environment: ClosureEnvironment::Captured(child.clone()),
        };
        child.insert("self".into(), &value).unwrap();
        parent.insert("f".into(), &value).unwrap();
        drop(value);
        drop(child);
        assert!(heap.lock().node(child_id).is_ok());
        if superassignment {
            caller.assign_parent("f".into(), &Sexp::Nil).unwrap();
        } else {
            parent.insert("f".into(), &Sexp::Nil).unwrap();
        }
        assert!(matches!(
            heap.lock().node(child_id),
            Err(GraphError::StaleNode)
        ));
        assert_eq!(parent.get("f").unwrap(), Some(Sexp::Nil));
    }
}
