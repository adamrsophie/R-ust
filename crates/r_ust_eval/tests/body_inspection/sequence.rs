use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{CallArgument, Sexp};

fn install(frontend: &mut Frontend) {
    frontend.eval_source("dispatch <- sequence").unwrap();
}

#[test]
fn genuine_sequence_dispatch_preserves_lookup_force_order_visibility_and_transfer() {
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
fn genuine_sequence_body_and_private_internal_boundary() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("body(sequence)"),
        Ok(vec![Sexp::Call {
            function: Box::new(Sexp::Symbol("UseMethod".into())),
            arguments: vec![CallArgument::positional(Sexp::String(vec![Some(
                "sequence".into()
            )]))],
        }])
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

#[test]
fn sequence_custom_method_and_alias_preserve_invisibility() {
    let mut frontend = Frontend::default();
    let result = frontend.eval_source("sequence.integer<-function(nvec,...)invisible(37L);p<-sequence;r<-withVisible((p)(1L));identical(r$value,37L)&&identical(r$visible,FALSE)").unwrap();
    assert_eq!(result.last(), Some(&Sexp::Logical(vec![Some(true)])));
    assert!(frontend.runtime_mut().take_output().is_empty());
}
