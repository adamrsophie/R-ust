use super::{streams, EvalError, Frontend, Sexp};
use crate::RuntimeState;
use r_ust_sexp::{ListElement, ListObject, PairListObject};

const PREFIX: &str = "f<-function(){on.exit({cat('FIRST|');quit(save='no',status=6L,runLast=FALSE);cat('AFTERQUIT|')},add=TRUE);on.exit(cat('SECOND|'),add=TRUE);";

#[test]
fn cleanup_quit_stops_every_remaining_action_and_unwinds_all_contexts() {
    let cases = [
        ("normal", "7L};f()"),
        ("return", "return(7L)};f()"),
        (
            "error",
            "stop('BODY')};tryCatch(f(),error=function(e)cat('HANDLER|'))",
        ),
        (
            "nested",
            "7L};outer<-function(){on.exit(cat('OUTER|'));f()};outer()",
        ),
        (
            "restart",
            "invokeRestart('r')};withRestarts(f(),r=function(){cat('RESTART_HANDLER|');0L})",
        ),
    ];
    for (id, suffix) in cases {
        check_quit(id, &format!("{PREFIX}{suffix}"));
    }
    check_quit("cleanup_restart", "f<-function(){on.exit(withRestarts({cat('FIRST|');quit(save='no',status=6L,runLast=FALSE);cat('AFTERQUIT|')},r=function()0L),add=TRUE);on.exit(cat('SECOND|'),add=TRUE);7L};f()");
}

fn check_quit(id: &str, source: &str) {
    let mut frontend = Frontend::default();
    let result = frontend.eval_source(source);
    assert!(
        matches!(result, Err(EvalError::Quit(ref request)) if request.status == 6 && !request.run_last),
        "{id}: {result:?}"
    );
    assert_eq!(
        streams(&mut frontend),
        ("FIRST|".into(), String::new()),
        "{id}"
    );
    assert!(frontend.runtime.conditions.head.is_none(), "{id}");
    assert!(frontend.runtime.conditions.restarts.is_none(), "{id}");
    assert!(frontend.runtime.control.contexts.is_empty(), "{id}");
    assert_eq!(
        frontend.eval_source("42L").unwrap().last(),
        Some(&Sexp::Integer(vec![Some(42)])),
        "{id}"
    );
}

#[test]
fn cleanup_quit_restores_existing_handler_and_restart_roots() {
    let mut runtime = RuntimeState::default();
    let retained = runtime.global.child().unwrap();
    retained
        .insert("payload".into(), &Sexp::Integer(vec![Some(41)]))
        .unwrap();
    let restart = root(
        &runtime,
        vec![
            Sexp::String(vec![Some("outer".into())]),
            Sexp::Environment(retained.clone()),
        ],
    );
    let result = crate::conditions::ordinary_list(&runtime.global, vec![Sexp::Nil; 4]).unwrap();
    let handler = root(
        &runtime,
        vec![
            Sexp::Char(Some("error".into())),
            Sexp::Environment(retained.clone()),
            Sexp::Integer(vec![Some(17)]),
            Sexp::Environment(retained.clone()),
            Sexp::ListObject(result),
        ],
    );
    runtime.conditions.restarts = Some(restart.clone());
    runtime.conditions.head = Some(handler.clone());
    drop(retained);
    let parsed = r_ust_parse::Parser.parse_all(
        "{f<-function(){on.exit(withCallingHandlers(withRestarts(quit(save='no',status=6L,runLast=FALSE),r=function()0L),warning=function(w)NULL),add=TRUE);on.exit(stop('UNREACHABLE'),add=TRUE);7L};f()}"
    ).unwrap();
    let result = runtime.eval(&parsed.expressions[0]);
    assert!(
        matches!(result, Err(EvalError::Quit(ref request)) if request.status == 6 && !request.run_last)
    );
    assert!(runtime.control.contexts.is_empty());
    assert!(runtime
        .conditions
        .head
        .as_ref()
        .unwrap()
        .same_object(&handler));
    assert!(runtime
        .conditions
        .restarts
        .as_ref()
        .unwrap()
        .same_object(&restart));
    assert!(runtime.take_output().is_empty());
    for root in [handler, restart] {
        let Sexp::ListObject(entry) = root.car().unwrap().value else {
            panic!("root");
        };
        let Sexp::Environment(environment) = entry.element(1).unwrap() else {
            panic!("retained environment");
        };
        assert_eq!(
            environment.get_local("payload").unwrap(),
            Some(Sexp::Integer(vec![Some(41)]))
        );
    }
}

fn root(runtime: &RuntimeState, values: Vec<Sexp>) -> PairListObject {
    let value = Sexp::List(
        values
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
    );
    let value = Sexp::ListObject(ListObject::from_value(&runtime.global, &value).unwrap());
    PairListObject::cons(&runtime.global, &ListElement { name: None, value }, None).unwrap()
}
