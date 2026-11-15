use super::{drained, evaluate, global, integer, last};
use r_ust_eval::{EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{CallArgument, Environment, Sexp};

#[test]
fn returned_closure_and_selected_environment_outlive_the_origin_runtime() {
    let closure = {
        let mut frontend = Frontend::default();
        let root = global(&mut frontend);
        let selected = root.child().unwrap();
        selected.insert("x".into(), &integer(73)).unwrap();
        frontend
            .runtime_mut()
            .eval(&evaluate(
                Sexp::Function {
                    parameters: vec![],
                    body: Box::new(Sexp::Symbol("x".into())),
                },
                &selected,
            ))
            .unwrap()
    };
    let mut destination = Frontend::default();
    last(&mut destination, "x<-99L");
    let value = destination
        .runtime_mut()
        .eval(&Sexp::Call {
            function: Box::new(closure),
            arguments: vec![],
        })
        .unwrap();
    assert_eq!(value, integer(73));
    assert_eq!(last(&mut destination, "x"), integer(99));
    drained(&mut destination);
}

#[test]
fn data_environment_identity_and_shared_values_survive_eval_unchanged() {
    let frame = Environment::global();
    let object = r_ust_sexp::ListObject::from_value(
        &frame,
        &Sexp::List(vec![r_ust_sexp::ListElement {
            name: Some("x".into()),
            value: integer(9),
        }]),
    )
    .unwrap();
    let mut runtime = RuntimeState::default();
    for value in [Sexp::Environment(frame.clone()), Sexp::ListObject(object)] {
        assert_eq!(
            runtime.eval(&evaluate(value.clone(), &frame)).unwrap(),
            value
        );
    }
}

fn failed_output(_: OutputEvent) -> std::io::Result<()> {
    let mut other = Frontend::default();
    assert_eq!(last(&mut other, "sys.nframe()"), integer(0));
    Err(std::io::Error::other("eval output failure"))
}

#[test]
fn host_output_failure_restores_selected_context_and_keeps_committed_mutation() {
    let mut frontend = Frontend::default();
    last(&mut frontend, "e<-new.env();e$x<-1L");
    frontend.runtime_mut().set_output_sink(failed_output);
    let error = frontend
        .eval_source("evalq({x<-2L;cat('tick');stop('UNREACHED')},e)")
        .unwrap_err();
    assert_eq!(error, EvalError::Output("eval output failure".into()));
    assert_eq!(last(&mut frontend, "e$x"), integer(2));
    assert_eq!(
        last(&mut frontend, "identical(environment(),globalenv())"),
        Sexp::Logical(vec![Some(true)])
    );
    drained(&mut frontend);
}

#[test]
fn internal_alias_eager_arguments_force_once_before_environment_validation() {
    let mut frontend = Frontend::default();
    last(&mut frontend, "f<-evalq(sys.function());n<-0L");
    let error = frontend
        .eval_source("f({n<-n+1L;quote(1L)},{n<-n+10L;TRUE},{n<-n+100L;2L})")
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "domain error: invalid 'enclos' argument of type 'NULL'"
    );
    assert_eq!(last(&mut frontend, "n"), integer(111));
    drained(&mut frontend);
}

#[test]
fn direct_internal_expression_uses_actual_selected_environment() {
    let mut frontend = Frontend::default();
    let frame = global(&mut frontend).child().unwrap();
    frame.insert("x".into(), &integer(61)).unwrap();
    let call = Sexp::InternalCall {
        operation: r_ust_sexp::InternalOperation::Eval,
        arguments: vec![
            CallArgument::positional(super::transported(Sexp::Symbol("x".into()))),
            CallArgument::positional(Sexp::Environment(frame)),
            CallArgument::positional(Sexp::Nil),
        ],
    };
    assert_eq!(frontend.runtime_mut().eval(&call).unwrap(), integer(61));
    drained(&mut frontend);
}
