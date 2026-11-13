use r_ust_eval::{EvalError, Frontend, OutputEvent};

#[test]
fn accepted_partial_warnings_precede_unused_forcing_and_follow_formal_order() {
    let mut frontend = Frontend::default();
    frontend.eval_source("options(warn=1,warnPartialMatchArgs=TRUE);g<-function(alpha,beta)NULL;f<-function(...)g(...)").unwrap();
    let result =
        frontend.eval_source("f(b=2L,al=1L,first={cat('FIRST');3L},second={cat('SECOND');4L})");
    assert_eq!(
        result,
        Err(EvalError::Arity(
            "unused arguments (first = 3, second = 4)".into()
        ))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stderr("Warning: partial argument match of 'al' to 'alpha'\n".into()),
            OutputEvent::Stderr("Warning: partial argument match of 'b' to 'beta'\n".into()),
            OutputEvent::Stdout("FIRST".into()),
            OutputEvent::Stdout("SECOND".into()),
        ]
    );
}
