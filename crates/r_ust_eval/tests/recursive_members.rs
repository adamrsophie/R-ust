use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;
use std::collections::HashSet;

#[test]
fn recursive_members_match_fresh_r_reference_checks() {
    let mut seen = HashSet::new();
    let mut failures = Vec::new();
    for row in include_str!("fixtures/recursive-members-r-4.6.0.tsv").lines() {
        if row.starts_with('#') || row.is_empty() {
            continue;
        }
        let (id, source) = row.split_once('\t').unwrap();
        assert!(seen.insert(id), "duplicate reference ID {id}");
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(source);
        if result.as_ref().ok().and_then(|v| v.last()) != Some(&Sexp::Logical(vec![Some(true)])) {
            failures.push(format!("{id}: {source}\n{result:?}"));
        }
    }
    assert_eq!(seen.len(), 89);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn recursive_sdk_members_preserve_unforced_payloads_and_copy_graph_roots() {
    use r_ust_sexp::{CallArgument, EvaluationOrigin, ListElement, ListObject, PromiseState};
    let mut frontend = Frontend::default();
    frontend.eval_source("hits<-0L").unwrap();
    let Sexp::Environment(owner) = frontend.eval_source("globalenv()").unwrap().pop().unwrap()
    else {
        panic!("global frame")
    };
    let expression = r_ust_parse::Parser
        .parse_all("{hits<<-hits+1L;7L}")
        .unwrap()
        .expressions
        .remove(0);
    let promise = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true).unwrap();
    let value = Sexp::List(vec![ListElement {
        name: None,
        value: Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::named(
                "payload",
                Sexp::Promise {
                    state: promise.clone(),
                },
            )],
        },
    }]);
    let original = ListObject::from_value(&owner, &value).unwrap();
    owner
        .insert("x".into(), &Sexp::ListObject(original.clone()))
        .unwrap();
    assert_eq!(
        frontend.eval_source("x[[c(1L,2L)]]").unwrap(),
        [Sexp::Promise {
            state: promise.clone()
        }]
    );
    assert_eq!(
        frontend.eval_source("hits").unwrap(),
        [Sexp::Integer(vec![Some(0)])]
    );
    frontend.eval_source("x[[c(1L,2L)]]<-quote(z)").unwrap();
    let Sexp::ListObject(updated) = frontend.runtime().get("x").unwrap().unwrap() else {
        panic!("explicit SDK list remains a graph object")
    };
    assert!(!updated.same_object(&original));
    assert_eq!(original.value().unwrap(), value);
    assert_eq!(
        frontend.eval_source("x[[c(1L,2L)]]").unwrap(),
        [Sexp::Symbol("z".into())]
    );
    drop(frontend);
    let mut consumer = r_ust_eval::RuntimeState::default();
    for _ in 0..2 {
        assert_eq!(
            consumer
                .eval(&Sexp::Promise {
                    state: promise.clone()
                })
                .unwrap(),
            Sexp::Integer(vec![Some(7)])
        );
    }
    assert_eq!(
        owner.get_local("hits").unwrap(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    drop(owner);
}
