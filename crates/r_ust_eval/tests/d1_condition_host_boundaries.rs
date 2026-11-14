use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fail_output(_: OutputEvent) -> std::io::Result<()> {
    Err(std::io::Error::other("closed output"))
}

fn fail_input() -> std::io::Result<String> {
    Err(std::io::Error::other("closed input"))
}

#[test]
fn public_error_boundary_restores_incoming_saved_head_after_host_failure() -> Result {
    for (effect, expected) in [
        ("cat('X')", EvalError::Output("closed output".into())),
        ("readline()", EvalError::Input("closed input".into())),
        (
            "q('no',7L,FALSE)",
            EvalError::Quit(r_ust_eval::QuitRequest {
                save: r_ust_eval::SaveAction::No,
                status: 7,
                run_last: false,
            }),
        ),
    ] {
        let mut runtime = Frontend::default();
        runtime.runtime_mut().set_session_context(vec![], true);
        let saved = runtime.eval_source(".Internal(.addCondHands('error',list(function(e)NULL),globalenv(),NULL,TRUE));.Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE))")?.pop().ok_or("incoming head")?;
        let Sexp::PairListObject(saved) = saved else {
            panic!("incoming ordinary head")
        };
        if matches!(&expected, EvalError::Output(_)) {
            runtime.runtime_mut().set_output_sink(fail_output);
        }
        runtime.runtime_mut().set_input_reader(fail_input);
        let body = format!(".Internal(.addCondHands('error',list(function(e)NULL),environment(),NULL,TRUE));{effect}");
        for source in [
            format!("{{{body}}}"),
            format!("f<-function(){{{body}}};f()"),
        ] {
            assert_eq!(
                runtime.eval_source(&source),
                Err(expected.clone()),
                "{source}"
            );
            let actual = runtime
                .eval_source(".Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE))")?
                .pop()
                .ok_or("restored head")?;
            let Sexp::PairListObject(actual) = actual else {
                panic!("restored ordinary head")
            };
            assert!(actual.same_object(&saved), "{effect}");
            assert_eq!(
                runtime.eval_source("sys.nframe()")?.pop(),
                Some(Sexp::Integer(vec![Some(0)]))
            );
        }
    }
    Ok(())
}

#[test]
fn successful_manual_reset_persists_at_public_evaluation_boundary() -> Result {
    let mut runtime = Frontend::default();
    let saved = runtime.eval_source(".Internal(.addCondHands('error',list(function(e)NULL),globalenv(),NULL,TRUE));saved<-.Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE));.Internal(.resetCondHands(NULL));saved")?.pop().ok_or("saved head")?;
    let Sexp::PairListObject(saved) = saved else {
        panic!("ordinary saved head")
    };
    runtime.eval_source(".Internal(.resetCondHands(saved))")?;
    let actual = runtime
        .eval_source(".Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE))")?
        .pop()
        .ok_or("persistent reset")?;
    let Sexp::PairListObject(actual) = actual else {
        panic!("ordinary reset head")
    };
    assert!(actual.same_object(&saved));
    Ok(())
}
