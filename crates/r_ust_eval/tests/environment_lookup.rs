use r_ust_eval::{EvalError, Frontend, RuntimeState};
use r_ust_parse::Parser;
use r_ust_sexp::{
    AssignmentOperator, CallArgument, ClosureEnvironment, Environment, EnvironmentRole, GraphError,
    Sexp,
};

#[path = "environment_lookup/candidates.rs"]
mod candidates;
#[path = "environment_lookup/namespace.rs"]
mod namespace;
#[path = "environment_lookup/source.rs"]
mod source;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}
fn expression(source: &str) -> Sexp {
    Parser.parse_all(source).unwrap().expressions.remove(0)
}
fn closure(frame: &Environment, body: Sexp) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(body),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}
fn invoke(runtime: &mut RuntimeState, function: &Sexp) -> Result<Sexp, EvalError> {
    runtime.eval(&Sexp::Call {
        function: Box::new(function.clone()),
        arguments: vec![],
    })
}
fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) {
    runtime
        .eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: name.into(),
            value: Box::new(value),
        })
        .unwrap();
}
fn root(runtime: &RuntimeState, role: EnvironmentRole) -> Environment {
    let Some(Sexp::Environment(global)) = runtime.get(".GlobalEnv").unwrap() else {
        panic!("global root missing")
    };
    global.session_root(role).unwrap()
}

#[test]
fn empty_and_missing() {
    let source = Environment::global();
    let empty = source.session_root(EnvironmentRole::Empty).unwrap();
    let child = empty.child().unwrap();
    let mut consumer = Frontend::default();
    for (code, name) in [
        ("sum({effect <<- 1L; 2L})", "sum"),
        ("abs({effect <<- 1L; -2L})", "abs"),
        (".Internal(effect <<- 1L)", ".Internal"),
        ("...length(effect <<- 1L)", "...length"),
        ("1L + (effect <<- 1L)", "+"),
        ("1L - (effect <<- 1L)", "-"),
        ("1L * (effect <<- 1L)", "*"),
        ("1L / (effect <<- 1L)", "/"),
        ("1L ^ (effect <<- 1L)", "^"),
        ("1L %% (effect <<- 1L)", "%%"),
        ("1L %/% (effect <<- 1L)", "%/%"),
        ("1L : (effect <<- 1L)", ":"),
        ("1L & (effect <<- 1L)", "&"),
        ("1L | (effect <<- 1L)", "|"),
        ("TRUE || (effect <<- 1L)", "||"),
        ("1L <= (effect <<- 1L)", "<="),
        ("1L > (effect <<- 1L)", ">"),
        ("1L >= (effect <<- 1L)", ">="),
        ("1L == (effect <<- 1L)", "=="),
        ("1L != (effect <<- 1L)", "!="),
        ("+(effect <<- 1L)", "+"),
        ("-(effect <<- 1L)", "-"),
        ("!(effect <<- 1L)", "!"),
        ("1L < (effect <<- 1L)", "<"),
        ("FALSE && (effect <<- 1L)", "&&"),
        ("1L %in% (effect <<- 1L)", "%in%"),
        ("(abs)(effect <<- 1L)", "("),
        ("(effect <<- 1L)", "("),
    ] {
        assert_eq!(
            invoke(consumer.runtime_mut(), &closure(&child, expression(code))),
            Err(EvalError::UndefinedFunction(name.into())),
            "{code}"
        );
        assert_eq!(child.local_len().unwrap(), 0);
        assert_eq!(source.get_local("effect").unwrap(), None);
        assert_eq!(consumer.runtime().get("effect").unwrap(), None);
    }
    assert_eq!(
        invoke(
            consumer.runtime_mut(),
            &closure(&child, Sexp::Symbol("identity".into()))
        ),
        Err(EvalError::UndefinedSymbol("identity".into()))
    );
    child
        .insert("sum".into(), &Sexp::Primitive(r_ust_sexp::PrimitiveId::Sum))
        .unwrap();
    assert_eq!(
        invoke(
            consumer.runtime_mut(),
            &closure(&child, expression("sum(2L, 3L)"))
        )
        .unwrap(),
        integer(5)
    );
    let local = closure(&child, integer(17));
    child.insert("local".into(), &local).unwrap();
    assert_eq!(
        invoke(
            consumer.runtime_mut(),
            &closure(&child, expression("local()"))
        )
        .unwrap(),
        integer(17)
    );
    assert_eq!(
        consumer.eval_source("sum(1L, 2L)").unwrap().pop(),
        Some(integer(3))
    );
}

#[test]
fn source_session() {
    source::source_session();
}

#[test]
fn retained_child_superassignment_uses_origin_global() {
    for role in [EnvironmentRole::Empty, EnvironmentRole::Base] {
        let (child, function) = {
            let source = RuntimeState::default();
            let parent = root(&source, role);
            let child = parent.child().unwrap();
            let function = closure(
                &child,
                Sexp::SuperAssignment {
                    target: "origin_only".into(),
                    value: Box::new(integer(41)),
                },
            );
            (Sexp::Environment(child), function)
        };
        let mut consumer = RuntimeState::default();
        bind(&mut consumer, "consumer_sentinel", integer(99));
        let Sexp::Environment(child) = child else {
            panic!("retained child is not an environment")
        };
        for value in [41, 42] {
            let mut current = function.clone();
            if let Sexp::Closure { body, .. } = &mut current {
                if let Sexp::SuperAssignment { value: slot, .. } = body.as_mut() {
                    **slot = integer(value);
                }
            }
            assert_eq!(invoke(&mut consumer, &current).unwrap(), integer(value));
            let origin = child.session_root(EnvironmentRole::Global).unwrap();
            assert_ne!(origin, root(&consumer, EnvironmentRole::Global));
            assert_eq!(
                origin.get_local("origin_only").unwrap(),
                Some(integer(value))
            );
            assert_eq!(consumer.get("origin_only").unwrap(), None);
            assert_eq!(
                consumer.get("consumer_sentinel").unwrap(),
                Some(integer(99))
            );
            assert_eq!(child.get_local("origin_only").unwrap(), None);
            for role in [
                EnvironmentRole::Empty,
                EnvironmentRole::Base,
                EnvironmentRole::BaseNamespace,
            ] {
                assert_eq!(
                    child
                        .session_root(role)
                        .unwrap()
                        .get_local("origin_only")
                        .unwrap(),
                    None
                );
            }
        }
    }
}

#[test]
fn retained_base_namespace_preserve_store_and_locks() {
    source::retained_store();
}
