use super::*;
use crate::{ClosureEnvironment, Sexp};

#[test]
fn dynamic_context_root_bundle_final_release() {
    let heap = Arc::new(Heap::default());
    let execution = heap.session().child().unwrap();
    let sysparent = heap.session().child().unwrap();
    let lexical = heap.session().child().unwrap();
    let ids = [execution.0.id(), sysparent.0.id(), lexical.0.id()];
    let callable = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(lexical.clone()),
    };
    let bundle = (execution.clone(), sysparent.clone(), callable);
    drop(execution);
    drop(sysparent);
    drop(lexical);
    for id in ids {
        let live = heap.lock().node(id).is_ok();
        assert!(live, "held context bundle must root {id:?}");
    }
    let exported = bundle.clone();
    drop(bundle);
    for id in ids {
        let live = heap.lock().node(id).is_ok();
        assert!(live, "exported alias must root {id:?}");
    }
    drop(exported);
    for id in ids {
        let stale = matches!(heap.lock().node(id), Err(GraphError::StaleNode));
        assert!(stale, "final drop must release {id:?}");
    }
    println!("execution/sysparent/callable roots: held live, alias live, final drop StaleNode");
}
