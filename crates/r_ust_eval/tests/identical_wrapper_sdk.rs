use r_ust_eval::{EvalError, Frontend, RuntimeState};
use r_ust_sexp::{CallArgument, ClosureEnvironment, FunctionParameter, Sexp};

#[test]
fn wrapper_is_a_real_closure_with_required_arguments_and_base_namespace() {
    let mut frontend = Frontend::default();
    let closure = frontend.eval_source("identical").unwrap().pop().unwrap();
    let Sexp::Closure {
        parameters,
        environment,
        ..
    } = closure
    else {
        panic!("identical must be a genuine source closure");
    };
    assert_eq!(parameters.len(), 9);
    assert_eq!(parameters[0], FunctionParameter::required("x"));
    assert_eq!(parameters[1], FunctionParameter::required("y"));
    for (parameter, (name, default)) in parameters[2..].iter().zip([
        ("num.eq", true),
        ("single.NA", true),
        ("attrib.as.set", true),
        ("ignore.bytecode", true),
        ("ignore.environment", false),
        ("ignore.srcref", true),
        ("extptr.as.ref", false),
    ]) {
        assert_eq!(parameter.name, name);
        assert_eq!(parameter.default, Some(Sexp::Logical(vec![Some(default)])));
    }
    let ClosureEnvironment::Captured(namespace) = environment else {
        panic!("missing source namespace");
    };
    assert_eq!(
        frontend.runtime().get(".BaseNamespaceEnv").unwrap(),
        Some(Sexp::Environment(namespace.clone()))
    );
    assert_ne!(
        RuntimeState::default().get(".BaseNamespaceEnv").unwrap(),
        Some(Sexp::Environment(namespace))
    );
}

#[test]
fn exported_identical_survives_the_originating_frontend() {
    let closure = Frontend::default()
        .eval_source("identical")
        .unwrap()
        .pop()
        .unwrap();
    let mut runtime = RuntimeState::default();
    for (left, right, expected) in [(7, 7, true), (7, 8, false)] {
        let expression = Sexp::Call {
            function: Box::new(closure.clone()),
            arguments: vec![
                CallArgument::positional(Sexp::Integer(vec![Some(left)])),
                CallArgument::positional(Sexp::Integer(vec![Some(right)])),
            ],
        };
        assert_eq!(
            runtime.eval(&expression).unwrap(),
            Sexp::Logical(vec![Some(expected)])
        );
        assert!(runtime.is_visible());
        assert!(runtime.take_output().is_empty());
    }
}

#[test]
fn existing_unimplemented_kernel_options_remain_explicit() {
    for option in [
        "num.eq=FALSE",
        "single.NA=FALSE",
        "ignore.environment=TRUE",
        "extptr.as.ref=TRUE",
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(&format!("identical(1L,1L,{option})")),
            Err(EvalError::NotImplemented(
                "nondefault identical comparison option"
            ))
        );
    }
}
