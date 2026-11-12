use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

fn strings(values: &[&str]) -> Sexp {
    Sexp::String(
        values
            .iter()
            .map(|value| Some((*value).to_string()))
            .collect(),
    )
}

#[test]
fn session_metadata_is_owned_by_each_host_runtime() {
    let mut first = Frontend::default();
    let mut second = Frontend::default();
    assert_eq!(
        first.eval_source("commandArgs();interactive()").unwrap(),
        vec![strings(&[]), Sexp::Logical(vec![Some(false)])]
    );
    first.runtime_mut().set_session_context(
        ["host", "--args", "α", "--args", ""]
            .map(str::to_string)
            .to_vec(),
        true,
    );
    assert_eq!(
        first
            .eval_source("commandArgs();commandArgs(TRUE);interactive()")
            .unwrap(),
        vec![
            strings(&["host", "--args", "α", "--args", ""]),
            strings(&["α", "--args", ""]),
            Sexp::Logical(vec![Some(true)]),
        ]
    );
    assert_eq!(
        second.eval_source("commandArgs();interactive()").unwrap(),
        vec![strings(&[]), Sexp::Logical(vec![Some(false)])]
    );
    first
        .runtime_mut()
        .set_session_context(vec!["other".to_string()], false);
    assert_eq!(
        first.eval_source("commandArgs(TRUE)").unwrap(),
        vec![strings(&[])]
    );
}

#[test]
fn command_args_matches_names_and_forces_only_matched_arguments() {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_session_context(
        ["host", "--args", "tail"].map(str::to_string).to_vec(),
        false,
    );
    for code in [
        "commandArgs(TRUE)",
        "commandArgs(tr=TRUE)",
        "commandArgs(trailingOnly=1)",
        "commandArgs(\"TRUE\")",
        "(function(...)commandArgs(...))(TRUE)",
        "lapply(TRUE,commandArgs)[[1]]",
    ] {
        assert_eq!(
            frontend.eval_source(code).unwrap(),
            vec![strings(&["tail"])],
            "{code}"
        );
    }
    for code in [
        "commandArgs(bogus=print(9))",
        "commandArgs(TRUE,print(9))",
        "commandArgs(tr=print(9),trailingOnly=TRUE)",
        "commandArgs(trailingOnly=TRUE,trailingOnly=print(9))",
    ] {
        assert!(
            matches!(frontend.eval_source(code), Err(EvalError::Arity(_))),
            "{code}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty(), "{code}");
    }
    assert_eq!(
        frontend
            .eval_source("(function()commandArgs(return(7)))()")
            .unwrap(),
        vec![Sexp::Real(vec![7.0])]
    );
    assert!(
        frontend
            .eval_source_visible("commandArgs(invisible(TRUE))")
            .unwrap()
            .len()
            == 1
    );
}

#[test]
fn conditions_require_one_nonmissing_logical_value() {
    for code in [
        "commandArgs(NULL)",
        "commandArgs(NA)",
        "commandArgs(c(TRUE,FALSE))",
        "if(NULL) 1",
        "if(c(TRUE,FALSE)) 1",
        "while(c(FALSE,TRUE)) 1",
    ] {
        assert!(
            matches!(
                Frontend::default().eval_source(code),
                Err(EvalError::Domain(_))
            ),
            "{code}"
        );
    }
    assert_eq!(
        Frontend::default()
            .eval_source("if(\"TRUE\") 1 else 2")
            .unwrap(),
        vec![Sexp::Real(vec![1.0])]
    );
}

#[test]
fn omitted_arguments_use_defaults_and_still_count_for_matching() {
    for code in [
        "(function(x=3)x)(x=)",
        "(function(x=3,y=4)x)(,4)",
        "(function(x=3,y=4)x)(,)",
        "(function(x=3,y=4)y)(1,)",
    ] {
        let expected = if code.ends_with("(1,)") { 4.0 } else { 3.0 };
        assert_eq!(
            Frontend::default().eval_source(code).unwrap(),
            vec![Sexp::Real(vec![expected])],
            "{code}"
        );
    }
    assert_eq!(
        Frontend::default()
            .eval_source("(function(x=3)missing(x))(x=)")
            .unwrap(),
        vec![Sexp::Logical(vec![Some(true)])]
    );
    assert!(
        matches!(Frontend::default().eval_source("(function(x)x)(x=)"), Err(EvalError::MissingArgument(name)) if name == "x")
    );
    assert!(matches!(
        Frontend::default().eval_source("commandArgs(, )"),
        Err(EvalError::Arity(_))
    ));
}
