use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{CallArgument, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn original_promises_are_shared_by_generic_and_method() -> Result {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("saved<-NULL;g<-function(x,...){saved<<-environment();UseMethod('g')};g.integer<-function(x,...)environment();g(1L,z=2L)")?;
    let Some(Sexp::Environment(method)) = values.last() else {
        panic!("method must return its live execution frame")
    };
    let Some(Sexp::Environment(generic)) = frontend.runtime().get("saved")? else {
        panic!("generic must retain its live execution frame")
    };
    let Some(Sexp::Promise { state: original }) = generic.get_local("x")? else {
        panic!("generic actual must remain a promise")
    };
    let Some(Sexp::Promise { state: selected }) = method.get_local("x")? else {
        panic!("method actual must remain a promise")
    };
    assert_eq!(
        original, selected,
        "dispatch must reuse the original promise handle"
    );
    assert_eq!(generic.get_local("...")?, method.get_local("...")?);
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.last(),
        Some(&Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}

#[test]
fn registered_method_and_result_outlive_origin_and_caller() -> Result {
    let generic = {
        let mut source = Frontend::default();
        source.eval_source("marker<-11L;tab<-new.env(parent=emptyenv());tab$g.integer<-function(x)function()c(x,marker);.__S3MethodsTable__.<-tab;function(x)UseMethod('g')")?
            .pop().ok_or("missing generic")?
    };
    let result = {
        let mut caller = RuntimeState::default();
        caller.eval(&Sexp::Call {
            function: Box::new(generic),
            arguments: vec![CallArgument::positional(Sexp::Integer(vec![Some(5)]))],
        })?
    };
    let mut final_runtime = RuntimeState::default();
    assert_eq!(
        final_runtime.eval(&Sexp::Call {
            function: Box::new(result),
            arguments: vec![]
        })?,
        Sexp::Integer(vec![Some(5), Some(11)])
    );
    Ok(())
}

fn fail_output(_: r_ust_eval::OutputEvent) -> std::io::Result<()> {
    Err(std::io::Error::other("dispatch output failed"))
}

fn fail_input() -> std::io::Result<String> {
    Err(std::io::Error::other("dispatch input failed"))
}

#[test]
fn method_host_failures_restore_next_evaluation() -> Result {
    for (method, expected) in [
        ("print.default(x)", "dispatch output failed"),
        ("readline()", "dispatch input failed"),
        ("unknown_dispatch_value", "unknown_dispatch_value"),
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source(&format!(
            "g<-function(x){{UseMethod('g');99L}};g.integer<-function(x){method}"
        ))?;
        if method == "print.default(x)" {
            frontend.runtime_mut().set_output_sink(fail_output);
        }
        frontend.runtime_mut().set_input_reader(fail_input);
        frontend.runtime_mut().set_session_context(vec![], true);
        let error = frontend
            .eval_source("g(1L)")
            .expect_err("method must propagate failure");
        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(
            frontend.eval_source("sys.nframe()")?.last(),
            Some(&Sexp::Integer(vec![Some(0)]))
        );
        assert_eq!(
            frontend
                .eval_source("g.integer<-function(x)17L;g(1L)")?
                .last(),
            Some(&Sexp::Integer(vec![Some(17)]))
        );
    }
    Ok(())
}
