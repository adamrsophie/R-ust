use super::*;
use crate::{ClosureEnvironment, Environment, PromiseState, Sexp};
#[test]
fn graph_concurrency_exports_survive_original_root_drop() {
    fn check<T: Send + Sync>() {}
    check::<Sexp>();
    check::<Environment>();
    check::<PromiseState>();
    let frame = Environment::global();
    let child = frame.child().unwrap();
    child
        .insert("n".into(), &Sexp::Integer(vec![Some(42)]))
        .unwrap();
    frame
        .insert(
            "f".into(),
            &Sexp::Closure {
                parameters: vec![],
                body: Box::new(Sexp::Nil),
                environment: ClosureEnvironment::Captured(child.clone()),
            },
        )
        .unwrap();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let ids = [frame.0.id(), child.0.id()];
    let barrier = Arc::new(std::sync::Barrier::new(5));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let frame = frame.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let value = frame.get("f").unwrap().unwrap();
                drop(frame);
                barrier.wait();
                barrier.wait();
                let Sexp::Closure {
                    environment: ClosureEnvironment::Captured(env),
                    ..
                } = value
                else {
                    panic!()
                };
                for _ in 0..100 {
                    assert_eq!(
                        env.clone().get("n").unwrap(),
                        Some(Sexp::Integer(vec![Some(42)]))
                    );
                }
            })
        })
        .collect();
    barrier.wait();
    drop(frame);
    drop(child);
    barrier.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
}
