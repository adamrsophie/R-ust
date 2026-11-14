use r_ust_eval::{Frontend, RuntimeState};
use r_ust_parse::Parser;
use r_ust_sexp::{
    AssignmentOperator, ClosureEnvironment, Environment, EnvironmentRole, EvaluationOrigin, Force,
    PrimitiveId, PromiseState, Sexp,
};

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) {
    runtime
        .eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: name.into(),
            value: Box::new(value),
        })
        .unwrap();
}
fn invoke(runtime: &mut RuntimeState, function: Sexp) -> Sexp {
    runtime
        .eval(&Sexp::Call {
            function: Box::new(function),
            arguments: vec![],
        })
        .unwrap()
}
fn source_closure(frame: &Environment, body: &str) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Parser.parse_all(body).unwrap().expressions.remove(0)),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn source_session_accessors_follow_captured_origin_and_bare_primitives_follow_receiver() {
    let mut consumer = RuntimeState::default();
    let receiver = invoke(&mut consumer, Sexp::Primitive(PrimitiveId::GlobalEnv));
    let (source, closures) = {
        let source_runtime = RuntimeState::default();
        let Some(Sexp::Environment(source)) = source_runtime.get(".GlobalEnv").unwrap() else {
            panic!("source root missing")
        };
        let child = source
            .session_root(EnvironmentRole::Empty)
            .unwrap()
            .child()
            .unwrap();
        for id in [
            PrimitiveId::GlobalEnv,
            PrimitiveId::BaseEnv,
            PrimitiveId::EmptyEnv,
        ] {
            child
                .insert(id.name().into(), &Sexp::Primitive(id))
                .unwrap();
        }
        let closures = [
            source_closure(&child, "globalenv()"),
            source_closure(&child, "baseenv()"),
            source_closure(&child, "emptyenv()"),
        ];
        (source, closures)
    };
    for ((closure, id), role) in closures
        .into_iter()
        .zip([
            PrimitiveId::GlobalEnv,
            PrimitiveId::BaseEnv,
            PrimitiveId::EmptyEnv,
        ])
        .zip([
            EnvironmentRole::Global,
            EnvironmentRole::Base,
            EnvironmentRole::Empty,
        ])
    {
        let expected = Sexp::Environment(source.session_root(role).unwrap());
        assert_eq!(invoke(&mut consumer, closure), expected);
        let bare = invoke(&mut consumer, Sexp::Primitive(id));
        assert_ne!(bare, expected);
        let Sexp::Environment(receiver) = &receiver else {
            panic!("receiver root missing")
        };
        assert_eq!(
            bare,
            Sexp::Environment(receiver.session_root(role).unwrap())
        );
    }
    let legacy = Sexp::Closure {
        parameters: vec![],
        body: Box::new(
            Parser
                .parse_all("globalenv()")
                .unwrap()
                .expressions
                .remove(0),
        ),
        environment: ClosureEnvironment::Base,
    };
    assert_eq!(invoke(&mut consumer, legacy), receiver);
    assert_ne!(receiver, Sexp::Environment(source));
}

#[test]
fn source_session_parent_name_and_predicate_never_force_frame_bindings() {
    let global = Environment::global();
    let child = global.child().unwrap();
    let state = PromiseState::new(
        &Sexp::Symbol("MUST_NOT_FORCE".into()),
        &child,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    child
        .insert(
            "pending".into(),
            &Sexp::Promise {
                state: state.clone(),
            },
        )
        .unwrap();
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(child.clone()),
    );
    assert_eq!(
        frontend
            .eval_source("parent.env(e);environmentName(e);is.environment(e)")
            .unwrap(),
        [
            Sexp::Environment(global),
            Sexp::String(vec![Some("".into())]),
            Sexp::Logical(vec![Some(true)]),
        ]
    );
    assert!(matches!(state.begin_force().unwrap(), Force::Evaluate(_)));
    assert_eq!(child.local_len().unwrap(), 1);
    assert_eq!(frontend.runtime_mut().take_output(), []);
}

#[test]
fn acquired_base_closures_keep_their_source_namespace_in_another_runtime() {
    let original = Frontend::default();
    let parent = original.runtime().get("parent.env").unwrap().unwrap();
    let name = original.runtime().get("environmentName").unwrap().unwrap();
    let Some(Sexp::Environment(global)) = original.runtime().get(".GlobalEnv").unwrap() else {
        panic!("source root missing")
    };
    drop(original);
    let mut receiver = Frontend::default();
    for (key, value) in [
        ("p", parent),
        ("n", name),
        ("g", Sexp::Environment(global.clone())),
    ] {
        bind(receiver.runtime_mut(), key, value);
    }
    assert_eq!(
        receiver.eval_source("p(g);n(p(g))").unwrap(),
        [
            Sexp::Environment(global.session_root(EnvironmentRole::Autoloads).unwrap()),
            Sexp::String(vec![Some("Autoloads".into())]),
        ]
    );
    let Some(Sexp::Closure { environment, .. }) = receiver.runtime().get("p").unwrap() else {
        panic!("parent closure missing")
    };
    assert_eq!(
        environment,
        ClosureEnvironment::Captured(global.session_root(EnvironmentRole::BaseNamespace).unwrap())
    );
}
