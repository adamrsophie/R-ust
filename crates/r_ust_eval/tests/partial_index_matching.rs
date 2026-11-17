use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;
use std::collections::HashSet;

#[test]
fn partial_index_matching_matches_gnu_values_and_warning_conditions() {
    let mut seen = HashSet::new();
    let mut failures = Vec::new();
    for row in include_str!("fixtures/partial-index-matching-r-4.6.0.tsv").lines() {
        if row.starts_with('#') || row.is_empty() {
            continue;
        }
        let (id, source) = row.split_once('\t').unwrap();
        assert!(seen.insert(id), "duplicate reference ID {id}");
        let result = Frontend::default().eval_source(source);
        if result.as_ref().ok().and_then(|values| values.last())
            != Some(&Sexp::Logical(vec![Some(true)]))
        {
            failures.push(format!("{id}: {source}\n{result:?}"));
        }
    }
    assert_eq!(seen.len(), 70);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn partial_matching_keeps_graph_payloads_unforced_and_read_roots_shared() {
    use r_ust_sexp::{CallArgument, EvaluationOrigin, ListElement, ListObject, PromiseState};
    let mut frontend = Frontend::default();
    frontend.eval_source("hits<-0L;options(warn=-1)").unwrap();
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
        name: Some("foobar".into()),
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
    for exact in ["FALSE", "NA"] {
        assert_eq!(
            frontend
                .eval_source(&format!("x[[c('foo','pay'),exact={exact}]]"))
                .unwrap(),
            [Sexp::Promise {
                state: promise.clone()
            }]
        );
        let Sexp::ListObject(current) = frontend.runtime().get("x").unwrap().unwrap() else {
            panic!("SDK root remains a graph object")
        };
        assert!(current.same_object(&original));
        assert_eq!(
            owner.get_local("hits").unwrap(),
            Some(Sexp::Integer(vec![Some(0)]))
        );
    }
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
}

#[test]
fn partial_matching_uses_effective_dimnames_and_preserves_array_attributes() {
    use r_ust_sexp::{AggregateVector, ListElement};
    fn attribute(name: &str, value: Sexp) -> ListElement {
        ListElement {
            name: Some(name.into()),
            value,
        }
    }
    let mut frontend = Frontend::default();
    frontend.eval_source("options(warn=-1)").unwrap();
    let Sexp::Environment(owner) = frontend.eval_source("globalenv()").unwrap().pop().unwrap()
    else {
        panic!("global frame")
    };
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![
            Sexp::List(vec![attribute("value", Sexp::Integer(vec![Some(1)]))]),
            Sexp::Nil,
        ]),
        attributes: vec![
            attribute("dim", Sexp::Integer(vec![Some(2)])),
            attribute(
                "dimnames",
                Sexp::Vector(vec![Sexp::String(vec![
                    Some("alphabet".into()),
                    Some("beta".into()),
                ])]),
            ),
            attribute("note", Sexp::Integer(vec![Some(5)])),
        ],
    };
    owner.insert("x".into(), &raw).unwrap();
    for exact in ["FALSE", "NA"] {
        assert_eq!(
            frontend
                .eval_source(&format!("x[[c('alph','val'),exact={exact}]]"))
                .unwrap(),
            [Sexp::Integer(vec![Some(1)])]
        );
    }
    assert_eq!(
        frontend.eval_source("before<-attributes(x);x[[c(1L,1L)]]<-7L;identical(attributes(x),before) && identical(x[[c('alph','val'),exact=FALSE]],7L)").unwrap().last(),
        Some(&Sexp::Logical(vec![Some(true)]))
    );
}
