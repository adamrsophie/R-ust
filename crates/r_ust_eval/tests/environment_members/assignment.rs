use super::*;
use r_ust_sexp::{ClosureEnvironment, EvaluationOrigin, ListElement, PromiseState};

#[test]
fn target_before_index() {
    let mut frontend = Frontend::default();
    let source = global(frontend.runtime());
    let frame = source.child().unwrap();
    source
        .insert("actual".into(), &Sexp::Environment(frame.clone()))
        .unwrap();
    let state = PromiseState::new(
        &expression("{cat('OBJECT');actual}"),
        &source,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    source
        .insert("target".into(), &Sexp::Promise { state })
        .unwrap();

    assert_eq!(
        frontend
            .eval_source("target[[{cat('INDEX');'x'}]]<-{cat('VALUE');7L}")
            .unwrap(),
        [integer(7)]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        ["VALUE", "OBJECT", "INDEX"].map(|value| OutputEvent::Stdout(value.into()))
    );
    assert_eq!(frame.get_local("x").unwrap(), Some(integer(7)));
    assert_eq!(frontend.eval_source("target$x").unwrap(), [integer(7)]);
    assert!(frontend.runtime_mut().take_output().is_empty());
}

#[test]
fn inherited_root_shadow_survives_selector_failure() {
    for (target, source, expected) in [
        (
            "e",
            "e[[{cat('INDEX');1L}]]<-{cat('VALUE');9L}",
            EvalError::Type("wrong args for environment subassignment".into()),
        ),
        (
            "holder",
            "holder[[{cat('INDEX');0L}]]$x<-{cat('VALUE');9L}",
            EvalError::Domain("extraction index must be a positive finite number".into()),
        ),
    ] {
        let frame = Environment::global().child().unwrap();
        frame.insert("x".into(), &integer(1)).unwrap();
        let mut frontend = Frontend::default();
        bind(
            frontend.runtime_mut(),
            "e",
            Sexp::Environment(frame.clone()),
        );
        frontend
            .eval_source(&format!(
                "holder<-list(e); f<-function(){{kept<<-function(){target};{source}}}"
            ))
            .unwrap();
        let original = frontend.runtime().get(target).unwrap().unwrap();

        assert_eq!(frontend.eval_source("f()"), Err(expected));
        let Some(Sexp::Closure {
            environment: ClosureEnvironment::Captured(local),
            ..
        }) = frontend.runtime().get("kept").unwrap()
        else {
            panic!("retained activation frame")
        };
        assert_eq!(
            local.get_local(target).unwrap(),
            Some(original.clone()),
            "{source}"
        );
        assert_eq!(frontend.runtime().get(target).unwrap(), Some(original));
        assert_eq!(frame.get_local("x").unwrap(), Some(integer(1)));
        assert_eq!(
            frontend.runtime_mut().take_output(),
            [
                OutputEvent::Stdout("VALUE".into()),
                OutputEvent::Stdout("INDEX".into())
            ]
        );
        assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
    }
}

#[test]
fn nested_outer_setter_uses_reevaluated_index_and_returns_container() {
    let first = Environment::global().child().unwrap();
    let second = first.child().unwrap();
    first.insert("x".into(), &integer(1)).unwrap();
    second.insert("x".into(), &integer(2)).unwrap();
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "h",
        Sexp::List(vec![
            ListElement {
                name: None,
                value: Sexp::Environment(first.clone()),
            },
            ListElement {
                name: None,
                value: Sexp::Environment(second.clone()),
            },
        ]),
    );
    frontend.eval_source("i<-0L").unwrap();

    frontend.eval_source("h[[{i<<-i+1L;i}]]$x<-9L").unwrap();
    assert_eq!(
        frontend.eval_source("h[[1L]];h[[2L]];i").unwrap(),
        [
            Sexp::Environment(first.clone()),
            Sexp::Environment(first.clone()),
            integer(2)
        ]
    );
    assert_eq!(first.get_local("x").unwrap(), Some(integer(9)));
    assert_eq!(second.get_local("x").unwrap(), Some(integer(2)));
}

#[test]
fn nested_named_outer_selector_rebinds_locally() {
    let frame = Environment::global().child().unwrap();
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(frame.clone()),
    );
    frontend
        .eval_source("h<-list(child=e);f<-function(){h$child[['x']]<-5L;function()h}")
        .unwrap();

    let values = frontend.eval_source("f()").unwrap();
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(local),
        ..
    } = &values[0]
    else {
        panic!("retained local holder")
    };
    assert_eq!(
        local.get_local("h").unwrap(),
        frontend.runtime().get("h").unwrap()
    );
    assert_eq!(frame.get_local("x").unwrap(), Some(integer(5)));
}

#[test]
fn nested_list_copy_on_write_is_an_explicit_boundary() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("h<-list(list(x=1L),list(x=2L));i<-0L")
        .unwrap();
    assert_eq!(
        frontend.eval_source("h[[{i<<-i+1L;i}]]$x<-9L"),
        Err(EvalError::NotImplemented(
            "nested non-environment member replacement"
        ))
    );
    assert_eq!(
        frontend.eval_source("h[[1L]]$x;h[[2L]]$x").unwrap(),
        [integer(1), integer(2)]
    );
    assert_eq!(
        frontend.eval_source("h[[1L]]$x$y<-9L"),
        Err(EvalError::NotImplemented(
            "member replacement deeper than two selectors"
        ))
    );
    assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
}

#[test]
fn member_assignment_returns_invisible_rhs_and_reads_are_visible() {
    let mut frontend = Frontend::default();
    frontend.eval_source("e<-globalenv();h<-list(e)").unwrap();
    for source in [
        "e$x<-7L",
        "e[['x']]<-7L",
        "h[[1L]]$x<-7L",
        "e$x<-invisible(7L)",
        "e[['x']]<-invisible(7L)",
        "h[[1L]]$x<-invisible(7L)",
    ] {
        assert_eq!(frontend.eval_source(source).unwrap(), [integer(7)]);
        assert!(!frontend.runtime().is_visible(), "{source}");
        assert_eq!(frontend.eval_source_visible("e$x").unwrap(), [integer(7)]);
    }
}
