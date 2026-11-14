use super::*;
use crate::{AssignmentOperator, CallArgument, ClosureEnvironment, Environment, ListElement, Sexp};

fn wrappers(value: Sexp) -> Vec<Sexp> {
    vec![
        Sexp::ClassedList {
            elements: vec![ListElement {
                name: Some("value".into()),
                value: value.clone(),
            }],
            class: Box::new(Sexp::String(vec![Some("container".into())])),
            class_first: false,
        },
        Sexp::ExtractArguments {
            value: Box::new(value.clone()),
            arguments: vec![],
        },
        Sexp::ExtractArguments {
            value: Box::new(Sexp::Nil),
            arguments: vec![CallArgument::named("exact", value.clone())],
        },
        Sexp::ReplacementAssignment {
            operator: AssignmentOperator::Left,
            target: Box::new(value.clone()),
            value: Box::new(Sexp::Nil),
        },
        Sexp::ReplacementAssignment {
            operator: AssignmentOperator::Left,
            target: Box::new(Sexp::Nil),
            value: Box::new(value),
        },
    ]
}

#[test]
fn d1_new_edges_release_cycles_after_last_export() {
    for index in 0..5 {
        let container = Environment::global();
        let capture = Environment::global();
        let heap = Arc::clone(container.0.heap().unwrap());
        let ids = [container.0.id(), capture.0.id()];
        let closure = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Environment(capture.clone())),
            environment: ClosureEnvironment::Captured(capture.clone()),
        };
        let value = wrappers(closure).remove(index);
        capture.insert("cycle".into(), &value).unwrap();
        container.insert("value".into(), &value).unwrap();
        drop(value);
        drop(capture);
        assert!(
            heap.lock().node(ids[1]).is_ok(),
            "new value edge must retain capture before export"
        );
        let exported = container.get("value").unwrap().unwrap();
        drop(container);
        assert!(heap.lock().node(ids[1]).is_ok());
        let alias = exported.clone();
        drop(exported);
        assert!(heap.lock().node(ids[1]).is_ok());
        drop(alias);
        for id in ids {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
    }
}

#[test]
fn d1_new_edges_reject_foreign_heap_without_partial_binding() {
    let private = Arc::new(Heap::default());
    let foreign = super::super::Environment(private.insert(Node::Frame(Frame::default())));
    let target = Environment::global();
    let sentinel = Sexp::Integer(vec![Some(7)]);
    target.insert("value".into(), &sentinel).unwrap();
    for value in wrappers(Sexp::Environment(foreign.clone())) {
        assert_eq!(
            target.insert("value".into(), &value),
            Err(GraphError::ForeignHeap)
        );
        assert_eq!(target.get("value").unwrap(), Some(sentinel.clone()));
    }
}
