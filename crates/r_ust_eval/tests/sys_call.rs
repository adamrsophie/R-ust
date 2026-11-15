use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};
#[path = "sys_call/errors.rs"]
mod errors;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn value(frontend: &mut Frontend, source: &str) -> Result<Sexp> {
    Ok(frontend
        .eval_source(source)?
        .pop()
        .ok_or("result missing")?)
}
fn call(name: &str) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: vec![],
    }
}
fn list(values: Vec<Sexp>) -> Sexp {
    Sexp::List(
        values
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
    )
}
fn truth(frontend: &mut Frontend, source: &str) -> Result {
    assert_eq!(value(frontend, source)?, Sexp::Logical(vec![Some(true)]));
    Ok(())
}
fn output(frontend: &mut Frontend) -> String {
    frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .map(|event| match event {
            OutputEvent::Stdout(text) => text,
            other => panic!("unexpected output {other:?}"),
        })
        .collect()
}

#[test]
fn sys_call_uses_the_real_base_wrapper_and_default() -> Result {
    let mut f = Frontend::default();
    truth(&mut f,"typeof(sys.call)=='closure' && identical(formals(sys.call),pairlist(which=0L)) && identical(body(sys.call),quote(.Internal(sys.call(which)))) && identical(environment(sys.call),environment(sys.frame))")
}

#[test]
fn sys_call_top_level_is_null_and_result_is_visible() -> Result {
    let mut f = Frontend::default();
    assert_eq!(value(&mut f, "sys.call()")?, Sexp::Nil);
    assert_eq!(value(&mut f, "sys.call(0L)")?, Sexp::Nil);
    truth(
        &mut f,
        "v<-withVisible(sys.call());is.null(v$value) && v$visible",
    )?;
    truth(&mut f,"f<-function()withVisible(sys.call({invisible(0L)}));v<-f();identical(v$value,quote(f())) && v$visible")
}

#[test]
fn sys_call_counts_absolute_and_relative_dynamic_function_frames() -> Result {
    let mut f = Frontend::default();
    let result=value(&mut f,"f<-function(){g<-function(){h<-function()list(sys.call(),sys.call(-1L),sys.call(-2L),sys.call(-3L),sys.call(1L),sys.call(2L),sys.call(3L));h()};g()};f()")?;
    assert_eq!(
        result,
        list(vec![
            call("h"),
            call("g"),
            call("f"),
            Sexp::Nil,
            call("f"),
            call("g"),
            call("h")
        ])
    );
    Ok(())
}

#[test]
fn sys_call_coerces_only_first_element_without_requiring_integer_type() -> Result {
    let mut f = Frontend::default();
    assert_eq!(value(&mut f,"f<-function()list(sys.call(0.9),sys.call(c(0L,999L)),sys.call('0'),sys.call(FALSE),sys.call(TRUE),sys.call(-0.9));f()")?,list(vec![call("f");6]));
    assert_eq!(output(&mut f), "");
    Ok(())
}

#[test]
fn sys_call_supplied_and_default_promises_select_their_actual_environments() -> Result {
    let mut f = Frontend::default();
    assert_eq!(
        value(&mut f, "f<-function(x){g<-function(y)y;g(x)};f(sys.call())")?,
        Sexp::Nil
    );
    assert_eq!(
        value(
            &mut f,
            "f<-function(x=sys.call()){g<-function(y)y;g(x)};f()"
        )?,
        call("f")
    );
    Ok(())
}

#[test]
fn sys_call_preserves_tags_missing_arguments_syntax_and_unforced_payloads() -> Result {
    let mut f = Frontend::default();
    truth(&mut f,"f<-function(...)sys.call();x<-f(alpha=1L+2L,,omega=stop('must not force'));identical(x,quote(f(alpha=1L+2L,,omega=stop('must not force'))))")?;
    truth(&mut f,"f<-function(x)sys.call();alias<-f;x<-alias(x={stop('must not force')});identical(x,quote(alias(x={stop('must not force')})))")?;
    truth(&mut f,"f<-function(...)g(...);g<-function(...)sys.call();x<-f(a=1L,b=stop('must not force'));identical(x,quote(g(...)))")
}

#[test]
fn sys_call_records_computed_and_do_call_heads() -> Result {
    let mut f = Frontend::default();
    truth(
        &mut f,
        "f<-function(x)sys.call();x<-(identity(f))(x=1L);identical(x,quote((identity(f))(x=1L)))",
    )?;
    truth(
        &mut f,
        "x<-do.call('f',list(x=1L));identical(x,quote(f(x=1L)))",
    )
}

#[test]
fn sys_call_direct_internal_excludes_its_own_current_frame() -> Result {
    let mut f = Frontend::default();
    assert_eq!(
        value(
            &mut f,
            "f<-function()list(.Internal(sys.call(0L)),sys.call());f()"
        )?,
        list(vec![Sexp::Nil, call("f")])
    );
    assert_eq!(value(&mut f,"f<-function(){g<-function()list(.Internal(sys.call(0L)),.Internal(sys.call(1L)),.Internal(sys.call()),.Internal(sys.call(a=0L,b=0L)));g()};f()")?,list(vec![call("f"),call("f"),Sexp::Nil,Sexp::Nil]));
    Ok(())
}

#[test]
fn sys_call_duplicate_does_not_mutate_the_active_call() -> Result {
    let mut f = Frontend::default();
    assert_eq!(
        value(
            &mut f,
            "f<-function(){x<-sys.call();x[[1L]]<-quote(other);list(x,sys.call())};f()"
        )?,
        list(vec![call("other"), call("f")])
    );
    Ok(())
}

#[test]
fn sys_call_partial_which_and_explicit_missing_use_wrapper_matching() -> Result {
    let mut f = Frontend::default();
    assert_eq!(
        value(
            &mut f,
            "f<-function()list(sys.call(),sys.call(w=0L),sys.call(which=0L),sys.call(which=));f()"
        )?,
        list(vec![call("f"); 4])
    );
    Ok(())
}

#[test]
fn sys_call_uses_loop_and_eval_contexts_without_changing_frame_rules() -> Result {
    let mut f = Frontend::default();
    assert_eq!(value(&mut f,"f<-function(){result<-NULL;for(i in 1L)result<-list(sys.call(),.Internal(sys.call(0L)));result};f()")?,list(vec![call("f"),Sexp::Nil]));
    truth(&mut f,"f<-function()eval(quote(sys.call()),environment());x<-f();identical(x,quote(eval(quote(sys.call()),environment())))")
}

#[test]
fn sys_call_forces_which_once_before_selecting_the_call() -> Result {
    let mut f = Frontend::default();
    assert_eq!(
        value(
            &mut f,
            "f<-function()sys.call(which={cat('WHICH|');0L});f()"
        )?,
        call("f")
    );
    assert_eq!(output(&mut f), "WHICH|");
    Ok(())
}
