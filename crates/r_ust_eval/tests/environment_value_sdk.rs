use r_ust_eval::{format_sexp, Frontend, RuntimeState};
use r_ust_sexp::{
    canonical_language, AssignmentOperator, CallArgument, ClosureEnvironment, Environment,
    EnvironmentRole, EvaluationOrigin, Force, LanguageError, ListElement, PromiseState, Sexp,
    SexpType,
};
#[path = "environment_value_sdk/lifetime.rs"]
mod lifetime;
#[path = "environment_value_sdk/metadata.rs"]
mod metadata;
#[path = "environment_value_sdk/printing.rs"]
mod printing;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}
fn list(name: Option<&str>, value: Sexp) -> Sexp {
    Sexp::List(vec![ListElement {
        name: name.map(str::to_string),
        value,
    }])
}
fn call(runtime: &mut RuntimeState, name: &str, values: Vec<Sexp>) -> Result<Sexp> {
    Ok(runtime.eval(&Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    })?)
}
#[test]
fn environment_value_observation_errors() -> Result {
    let frame = Environment::global().child()?;
    let pending = PromiseState::new(
        &Sexp::Symbol("must_not_force".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    frame.insert(
        "lazy".into(),
        &Sexp::Promise {
            state: pending.clone(),
        },
    )?;
    let value = Sexp::Environment(frame.clone());
    assert_eq!(value.sexptype(), SexpType::Environment);
    assert_eq!(canonical_language(&value), Err(LanguageError::RuntimeValue));
    assert_eq!(
        canonical_language(&list(None, value.clone())),
        Err(LanguageError::RuntimeValue)
    );
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), "e", value.clone())?;
    for name in ["typeof", "class", "mode"] {
        assert_eq!(
            call(frontend.runtime_mut(), name, vec![value.clone()])?,
            Sexp::String(vec![Some("environment".into())])
        );
    }
    assert_eq!(
        call(frontend.runtime_mut(), "length", vec![value.clone()])?,
        Sexp::Integer(vec![Some(1)])
    );
    assert_eq!(frame.name()?, "");
    assert!(format_sexp(&value)?.starts_with("<environment: 0x"));
    assert_eq!(
        call(
            frontend.runtime_mut(),
            "identical",
            vec![value.clone(), value.clone()]
        )?,
        Sexp::Logical(vec![Some(true)])
    );
    let other = Sexp::Environment(Environment::global().child()?);
    assert_eq!(
        call(frontend.runtime_mut(), "identical", vec![value, other])?,
        Sexp::Logical(vec![Some(false)])
    );
    for (source, expected) in [
        (
            "as.integer(e)",
            "cannot coerce type 'environment' to vector of type 'integer'",
        ),
        (
            "as.double(e)",
            "cannot coerce type 'environment' to vector of type 'double'",
        ),
        (
            "as.character(e)",
            "cannot coerce type 'environment' to vector of type 'character'",
        ),
        (
            "as.logical(e)",
            "cannot coerce type 'environment' to vector of type 'logical'",
        ),
        ("e+1L", "non-numeric argument to binary operator"),
        ("-e", "invalid argument to unary operator"),
        ("!e", "invalid argument type"),
        ("sum(e)", "invalid 'type' (environment) of argument"),
        ("vector('integer',e)", "invalid 'length' argument"),
    ] {
        let error = frontend.eval_source(source).expect_err(source);
        assert_eq!(
            error
                .to_string()
                .split_once(": ")
                .expect("error category")
                .1,
            expected,
            "{source}"
        );
    }
    assert!(matches!(pending.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn environment_value_identity_is_not_binding_equality() -> Result {
    let a = Environment::global().child()?;
    let b = Environment::global().child()?;
    let mut runtime = RuntimeState::default();
    for value in [Sexp::Nil, Sexp::Integer(vec![Some(4)])] {
        a.insert("same".into(), &value)?;
        b.insert("same".into(), &value)?;
        let a = Sexp::Environment(a.clone());
        let b = Sexp::Environment(b.clone());
        for (left, right) in [(a.clone(), b.clone()), (list(None, a), list(None, b))] {
            assert_eq!(
                call(&mut runtime, "identical", vec![left, right])?,
                Sexp::Logical(vec![Some(false)])
            );
        }
    }
    Ok(())
}
