use r_ust_eval::Frontend;
use r_ust_sexp::{ClosureEnvironment, Environment, EvaluationOrigin, PromiseState, Sexp};

#[test]
fn foreign_global_promises_cannot_target_destination_global_loops() {
    for signal in ["break", "next"] {
        let mut same_session = Frontend::default();
        let same = same_session
            .eval_source(&format!(
                "f<-function(x)function()x;p<-f({signal});for(i in 1L){{p();cat('BAD')}};42L"
            ))
            .unwrap();
        assert_eq!(same.last(), Some(&Sexp::Integer(vec![Some(42)])));
        assert!(same_session.runtime_mut().take_output().is_empty());
        let mut source = Frontend::default();
        let escaped = source
            .eval_source(&format!("f<-function(x)function()x;f({signal})"))
            .unwrap()
            .pop()
            .unwrap();
        let mut destination = Frontend::default();
        destination
            .runtime_mut()
            .eval(&Sexp::Assignment {
                operator: r_ust_sexp::AssignmentOperator::Left,
                target: "foreign".into(),
                value: Box::new(escaped),
            })
            .unwrap();
        drop(source);
        let error = destination
            .eval_source("for(i in 1L){foreign();cat('BAD')};99L")
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "control-flow error: no loop for break/next, jumping to top level"
        );
        assert!(destination.runtime_mut().take_output().is_empty());
        assert_eq!(
            destination
                .eval_source("for(i in 1L)break;42L")
                .unwrap()
                .pop(),
            Some(Sexp::Integer(vec![Some(42)]))
        );
    }
}

#[test]
fn legacy_global_host_origin_is_detached_from_runtime_loops() {
    for expression in [Sexp::Break, Sexp::Next] {
        let frame = Environment::global();
        let state =
            PromiseState::new(&expression, &frame, EvaluationOrigin::default(), true).unwrap();
        let closure = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Promise { state }),
            environment: ClosureEnvironment::Base,
        };
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .eval(&Sexp::Assignment {
                operator: r_ust_sexp::AssignmentOperator::Left,
                target: "host".into(),
                value: Box::new(closure),
            })
            .unwrap();
        let error = frontend.eval_source("for(i in 1L)host();99L").unwrap_err();
        assert_eq!(
            error.to_string(),
            "control-flow error: no loop for break/next, jumping to top level"
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}
