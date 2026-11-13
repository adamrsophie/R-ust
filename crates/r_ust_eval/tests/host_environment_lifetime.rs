use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{CallArgument, Sexp};

fn call(runtime: &mut RuntimeState, function: &Sexp) -> Sexp {
    runtime
        .eval(&Sexp::Call {
            function: Box::new(function.clone()),
            arguments: vec![],
        })
        .unwrap()
}

#[test]
fn retained_counter_outlives_originating_frontend_and_runtimes_are_isolated() {
    let function = {
        let mut frontend = Frontend::default();
        frontend
            .eval_source("x<-11L;make<-function(){n<-0L;function(){n<<-n+1L;c(n,x)}};make()")
            .unwrap()
            .pop()
            .unwrap()
    };
    let mut other = Frontend::default();
    other.eval_source("x<-99L").unwrap();
    assert_eq!(
        call(other.runtime_mut(), &function),
        Sexp::Integer(vec![Some(1), Some(11)])
    );
    assert_eq!(
        call(other.runtime_mut(), &function),
        Sexp::Integer(vec![Some(2), Some(11)])
    );
    assert_eq!(
        other.runtime().get("x").unwrap(),
        Some(Sexp::Integer(vec![Some(99)]))
    );
    other
        .runtime_mut()
        .eval(&Sexp::Assignment {
            target: "foreign".into(),
            value: Box::new(function.clone()),
        })
        .unwrap();
    assert_eq!(
        other.eval_source("foreign()").unwrap().pop(),
        Some(Sexp::Integer(vec![Some(3), Some(11)]))
    );
}

#[test]
fn exchanged_closures_keep_each_lexical_global_and_dead_origins_stay_dead() {
    let mut left = Frontend::default();
    let mut right = Frontend::default();
    let left_fn = left
        .eval_source("x<-1L;function()x")
        .unwrap()
        .pop()
        .unwrap();
    let right_fn = right
        .eval_source("x<-2L;function()x")
        .unwrap()
        .pop()
        .unwrap();
    left.runtime_mut()
        .eval(&Sexp::Assignment {
            target: "foreign".into(),
            value: Box::new(right_fn.clone()),
        })
        .unwrap();
    right
        .runtime_mut()
        .eval(&Sexp::Assignment {
            target: "foreign".into(),
            value: Box::new(left_fn.clone()),
        })
        .unwrap();
    assert_eq!(
        left.eval_source("foreign()").unwrap().pop(),
        Some(Sexp::Integer(vec![Some(2)]))
    );
    assert_eq!(
        right.eval_source("foreign()").unwrap().pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    let escaped = left
        .eval_source("(function(){f<-function(x)function()x;f(return(8L))})()")
        .unwrap()
        .pop()
        .unwrap();
    drop(left);
    let error = right
        .runtime_mut()
        .eval(&Sexp::Call {
            function: Box::new(escaped),
            arguments: vec![],
        })
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "control-flow error: no function to return from, jumping to top level"
    );
    assert_eq!(
        call(right.runtime_mut(), &left_fn),
        Sexp::Integer(vec![Some(1)])
    );
}

#[test]
fn discarded_calls_do_not_change_retained_counter_state() {
    let mut frontend = Frontend::default();
    let factory = frontend
        .eval_source("function(){n<-0L;function(){n<<-n+1L;n}}")
        .unwrap()
        .pop()
        .unwrap();
    let counter = call(frontend.runtime_mut(), &factory);
    for _ in 0..10_000 {
        drop(call(frontend.runtime_mut(), &factory));
    }
    assert_eq!(
        call(frontend.runtime_mut(), &counter),
        Sexp::Integer(vec![Some(1)])
    );
    assert_eq!(
        call(frontend.runtime_mut(), &counter),
        Sexp::Integer(vec![Some(2)])
    );
    let identity = frontend.eval_source("function(x)x").unwrap().pop().unwrap();
    let supplied = Sexp::Call {
        function: Box::new(identity),
        arguments: vec![CallArgument::positional(counter)],
    };
    let retained = frontend.runtime_mut().eval(&supplied).unwrap();
    drop(supplied);
    drop(factory);
    drop(frontend);
    assert_eq!(
        call(&mut RuntimeState::default(), &retained),
        Sexp::Integer(vec![Some(3)])
    );
}
