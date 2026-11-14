use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{CallArgument, ListElement, Sexp};

#[path = "dynamic_frames/mod.rs"]
mod dynamic_frames;

fn value(source: &str) -> Sexp {
    Frontend::default()
        .eval_source(source)
        .unwrap()
        .pop()
        .unwrap()
}

#[test]
fn dynamic_caller_differs_from_lexical_parent() {
    assert_eq!(
        value("f<-function(){g<-function()identical(parent.frame(),environment());g()};f()"),
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        value("f<-function(){lex<-environment();g<-function()parent.frame();h<-function(){caller<-environment();c(identical(g(),caller),identical(g(),lex))};h()};f()"),
        Sexp::Logical(vec![Some(true), Some(false)])
    );
}

#[test]
fn environment_null_returns_wrapper_caller() {
    assert_eq!(value("f<-function(){e<-environment();g<-function()c(identical(parent.frame(),e),identical(environment(NULL),e));g()};f()"),
        Sexp::Logical(vec![Some(true), Some(false)]));
}

#[test]
fn new_env_default_parent_is_callers_frame() {
    assert_eq!(
        value("f<-function(){e<-environment();identical(parent.env(new.env()),e)};f()"),
        Sexp::Logical(vec![Some(true)])
    );
}

#[test]
fn sys_frames_positive_negative_and_zero() {
    assert_eq!(value("f<-function(){a<-environment();g<-function(){b<-environment();h<-function()c(identical(sys.frame(0L),globalenv()),identical(sys.frame(1L),a),identical(sys.frame(2L),b),identical(sys.frame(-1L),b),identical(sys.frame(-2L),a),identical(sys.frame(-3L),globalenv()));h()};g()};f()"),
        Sexp::Logical(vec![Some(true); 6]));
}

#[test]
fn sys_parent_zero_negative_and_saturation() {
    assert_eq!(value("f<-function(){g<-function()c(sys.parent(0L),sys.parent(-3L),sys.parent(1L),sys.parent(2L),sys.parent(2147483647L));g()};f()"),
        Sexp::Integer(vec![Some(2), Some(2), Some(1), Some(0), Some(0)]));
}

#[test]
fn sys_function_preserves_invoked_callable() {
    assert_eq!(
        value("f<-function()identical(sys.function(),f);alias<-f;c(alias(),(identity(alias))())"),
        Sexp::Logical(vec![Some(true), Some(true)])
    );
}

#[test]
fn formals_default_selects_calling_function() {
    assert_eq!(
        value("f<-function(x=19L,y=NULL)formals();f()"),
        Sexp::pairlist(vec![
            ListElement {
                name: Some("x".into()),
                value: Sexp::Integer(vec![Some(19)])
            },
            ListElement {
                name: Some("y".into()),
                value: Sexp::Nil
            },
        ])
    );
}

#[test]
fn body_default_selects_calling_function() {
    assert_eq!(
        value("f<-function(x=1L)body();f()"),
        Sexp::Call {
            function: Box::new(Sexp::Symbol("body".into())),
            arguments: Vec::<CallArgument>::new(),
        }
    );
}

#[test]
fn promise_supplied_and_default_frame_selection() {
    for (source, expected) in [
        ("f<-function(x){g<-function(y)y;g(x)};f(c(identical(parent.frame(),globalenv()),identical(environment(),globalenv()),sys.nframe(),sys.parent()))", vec![1,1,0,0]),
        ("f<-function(x=c(sys.nframe(),sys.parent())){g<-function(y)y;g(x)};f()", vec![1,0]),
        ("f<-function(x=c(sys.nframe(),sys.parent())){g<-function(y)y;g(x)};print(f())", vec![2,0]),
        ("f<-function(x){g<-function(y)y;g(x)};print(f(c(sys.nframe(),sys.parent())))", vec![0,0]),
    ] {
        assert_eq!(value(source), Sexp::Integer(expected.into_iter().map(Some).collect()), "{source}");
    }
}

#[test]
fn new_env_force_order_and_hash_size_coercion() {
    let mut frontend = Frontend::default();
    assert_eq!(frontend.eval_source("is.environment(new.env(hash={cat('HASH\n');FALSE},parent={cat('PARENT\n');globalenv()},size={cat('SIZE\n');'bad'}))").unwrap(),
        [Sexp::Logical(vec![Some(true)])]);
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("HASH\n".into()),
            OutputEvent::Stdout("PARENT\n".into()),
            OutputEvent::Stdout("SIZE\n".into())
        ]
    );
    assert_eq!(
        value("is.environment(new.env(hash=FALSE,size=list(list(1L))))"),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        frontend.eval_source("new.env(hash=FALSE,size=stop('SIZE STOP'))"),
        Err(EvalError::Stop {
            message: "SIZE STOP".into(),
            call_requested: true,
        })
    );
    assert_eq!(
        frontend.eval_source("sys.nframe();42L").unwrap(),
        [Sexp::Integer(vec![Some(0)]), Sexp::Integer(vec![Some(42)])]
    );
}

#[test]
fn new_env_hash_coerce_before_parent_validation() {
    assert_eq!(
        Frontend::default().eval_source("new.env(hash='bad',parent=NULL,size='bad')"),
        Err(EvalError::WithWarnings {
            error: Box::new(EvalError::Domain(
                "use of NULL environment is defunct".into()
            )),
            warnings: "Warning message:\nNAs introduced by coercion \n".into(),
        })
    );
}

#[test]
fn direct_internal_variadic_arity_and_tags() {
    let mut frontend = Frontend::default();
    for source in [
        ".Internal(sys.parent())",
        ".Internal(sys.parent(a=0L,b=0L))",
        ".Internal(sys.nframe())",
        ".Internal(sys.nframe(a=0L,b=0L))",
    ] {
        assert_eq!(
            frontend.eval_source(source).unwrap(),
            [Sexp::Integer(vec![Some(0)])]
        );
    }
    for source in [
        ".Internal(sys.frame())",
        ".Internal(sys.function())",
        ".Internal(sys.frame(0L,0L))",
        ".Internal(sys.function(0L,0L))",
    ] {
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::Domain(
                "not that many frames on the stack".into()
            ))
        );
    }
    assert_eq!(
        value("identical(.Internal(sys.frame(unused=0L)),globalenv())"),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        frontend
            .eval_source(".Internal(sys.nframe('bad'))")
            .unwrap(),
        [Sexp::Integer(vec![Some(0)])]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stderr(
            "Warning message:\nNAs introduced by coercion \n".into()
        )]
    );
}

#[test]
fn all_public_dynamic_frame_fixture_rows() {
    dynamic_frames::fixture::run_original();
}

#[test]
fn all_supported_dynamic_frame_source_fixture_rows() {
    dynamic_frames::fixture::run_supplemental();
}
