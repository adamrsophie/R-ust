use super::*;
use crate::{ClosureEnvironment, Environment, EnvironmentRole, PromiseState, Sexp};
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

#[test]
fn environment_bootstrap_concurrent_publish_preserves_one_complete_inventory() {
    let global = Environment::global();
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let threads: Vec<_> = [11, 29]
        .into_iter()
        .map(|value| {
            let global = global.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let bindings = [
                    ("first".into(), Sexp::Integer(vec![Some(value)])),
                    ("second".into(), Sexp::Integer(vec![Some(value)])),
                ];
                barrier.wait();
                global.install_base(&bindings).unwrap();
            })
        })
        .collect();
    barrier.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    for role in [EnvironmentRole::Base, EnvironmentRole::BaseNamespace] {
        let root = global.session_root(role).unwrap();
        let first = root.get_local("first").unwrap();
        assert!(
            first == Some(Sexp::Integer(vec![Some(11)]))
                || first == Some(Sexp::Integer(vec![Some(29)]))
        );
        assert_eq!(root.get_local("second").unwrap(), first);
        assert_eq!(root.local_len().unwrap(), 2);
    }
}
