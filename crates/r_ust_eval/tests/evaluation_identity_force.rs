#[path = "support/evaluation_identity_force.rs"]
mod reference;

use r_ust_eval::{Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{CallArgument, ClosureEnvironment, FunctionParameter, Sexp};

#[test]
fn identity_matches_independent_r_values_errors_visibility_and_effects() {
    reference::check("identity");
}

#[test]
fn force_matches_independent_r_values_errors_visibility_and_effects() {
    reference::check("force");
}

#[test]
fn evaluation_functions_are_base_closures_with_genuine_formals_and_symbol_body() {
    for name in ["identity", "force"] {
        let mut frontend = Frontend::default();
        let value = frontend.eval_source(name).unwrap().pop().unwrap();
        let Sexp::Closure {
            parameters,
            body,
            environment,
        } = value
        else {
            panic!("{name} is not a closure");
        };
        assert_eq!(parameters, vec![FunctionParameter::required("x")]);
        assert_eq!(*body, Sexp::Symbol("x".into()));
        let ClosureEnvironment::Captured(namespace) = environment else {
            panic!("{name} has no session namespace");
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
}

#[test]
fn forced_and_lazy_captured_closures_survive_the_originating_frontend() {
    for name in ["identity", "force"] {
        for force_before_capture in [false, true] {
            let mut frontend = Frontend::default();
            let source = if force_before_capture {
                format!("make<-function(x){{{name}(x);function()x}};make({{cat('CAPTURE');8L}})")
            } else {
                format!("make<-function(x){name}(function()x);make({{cat('CAPTURE');8L}})")
            };
            let closure = frontend.eval_source(&source).unwrap().pop().unwrap();
            let origin_output = frontend.runtime_mut().take_output();
            assert_eq!(
                origin_output,
                if force_before_capture {
                    vec![OutputEvent::Stdout("CAPTURE".into())]
                } else {
                    vec![]
                }
            );
            drop(frontend);
            let mut runtime = RuntimeState::default();
            let call = Sexp::Call {
                function: Box::new(closure),
                arguments: vec![],
            };
            for iteration in 0..2 {
                assert_eq!(runtime.eval(&call).unwrap(), Sexp::Integer(vec![Some(8)]));
                let output = runtime.take_output();
                assert_eq!(
                    output,
                    if force_before_capture || iteration == 1 {
                        vec![]
                    } else {
                        vec![OutputEvent::Stdout("CAPTURE".into())]
                    }
                );
            }
        }
    }
}

#[test]
fn host_can_retain_base_closure_and_supply_values_after_frontend_drop() {
    for name in ["identity", "force"] {
        let closure = Frontend::default()
            .eval_source(name)
            .unwrap()
            .pop()
            .unwrap();
        let mut runtime = RuntimeState::default();
        let value = Sexp::Integer(vec![Some(17), None]);
        let call = Sexp::Call {
            function: Box::new(closure),
            arguments: vec![CallArgument::named("x", value.clone())],
        };
        assert_eq!(runtime.eval(&call).unwrap(), value);
        assert!(runtime.is_visible());
    }
}
