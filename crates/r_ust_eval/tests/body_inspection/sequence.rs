use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{InternalOperation, Sexp};

fn install(frontend: &mut Frontend) {
    let mut closure = frontend
        .eval_source("function(nvec, ...) NULL")
        .unwrap()
        .pop()
        .unwrap();
    let Sexp::Closure { body, .. } = &mut closure else {
        panic!("expected closure");
    };
    **body = Sexp::InternalCall {
        operation: InternalOperation::SequenceDispatch,
        arguments: vec![],
    };
    let Some(Sexp::Environment(global)) = frontend.runtime().get(".GlobalEnv").unwrap() else {
        panic!("expected global");
    };
    global.insert("dispatch".into(), &closure).unwrap();
}

#[test]
fn synthetic_sequence_dispatch_preserves_lookup_force_order_visibility_and_transfer() {
    let mut frontend = Frontend::default();
    install(&mut frontend);
    frontend
        .eval_source("sequence.default <- function(...) { cat('DISPATCH'); invisible(9L) }")
        .unwrap();
    assert_eq!(
        frontend
            .eval_source("dispatch({cat('NVEC');1L},stop('UNFORCED'))")
            .unwrap(),
        [Sexp::Integer(vec![Some(9)])]
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("NVEC".into()),
            OutputEvent::Stdout("DISPATCH".into())
        ]
    );
    assert_eq!(
        frontend.eval_source("dispatch(stop('NVEC'),stop('DOTS'))"),
        Err(EvalError::Stop {
            message: "NVEC".into(),
            call_requested: true
        })
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
    frontend
        .eval_source("sequence.default <- function(...) 12L")
        .unwrap();
    assert_eq!(
        frontend.eval_source("dispatch(1L)").unwrap(),
        [Sexp::Integer(vec![Some(12)])]
    );
    assert!(frontend.runtime().is_visible());
    assert_eq!(
        frontend
            .eval_source("(function(){dispatch(return(17L));99L})()")
            .unwrap(),
        [Sexp::Integer(vec![Some(17)])]
    );
    assert_eq!(
        frontend
            .eval_source("(function(){for(i in 1L){dispatch(break);stop('BAD')};18L})()")
            .unwrap(),
        [Sexp::Integer(vec![Some(18)])]
    );
    assert_eq!(
        frontend
            .eval_source("(function(){for(i in 1L){dispatch(next);stop('BAD')};19L})()")
            .unwrap(),
        [Sexp::Integer(vec![Some(19)])]
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
}

#[test]
fn synthetic_sequence_body_has_an_explicit_source_boundary() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("body(sequence)"),
        Err(EvalError::NotImplemented(
            "function body source representation"
        ))
    );
    assert_eq!(
        frontend.eval_source(".Internal(sequenceDispatch())"),
        Err(EvalError::Domain(
            "there is no .Internal function 'sequenceDispatch'".into()
        ))
    );
    assert_eq!(
        frontend.eval_source("sequence(c(2L,1L))").unwrap(),
        [Sexp::Integer(vec![Some(1), Some(2), Some(1)])]
    );
}
