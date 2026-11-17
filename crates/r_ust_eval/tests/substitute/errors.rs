use super::*;

#[test]
fn matching_errors_precede_env_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    for (source, expected) in [
        (
            "substitute(x,1L)",
            EvalError::Type("invalid environment specified".into()),
        ),
        (
            "substitute(expr=x,expr=y,env=stop('ENV'))",
            EvalError::Arity(
                "formal argument \"expr\" matched by multiple actual arguments".into(),
            ),
        ),
        (
            "substitute(x,{cat('ENV');NULL},z=1L)",
            EvalError::Arity("unused argument (z = 1)".into()),
        ),
        (
            "substitute(x,NULL,)",
            EvalError::Arity("unused argument (alist())".into()),
        ),
        (
            "substitute(x,NULL,z=1L+2L)",
            EvalError::Arity("unused argument (z = 1 + 2)".into()),
        ),
        (
            "substitute(e=x)",
            EvalError::Arity("argument 1 matches multiple formal arguments".into()),
        ),
        (
            "substitute(f(...),list(...=1L))",
            EvalError::Domain("'...' used in an incorrect context".into()),
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(frontend.eval_source(source), Err(expected), "{source}");
        assert!(frontend.runtime_mut().take_output().is_empty(), "{source}");
    }
    Ok(())
}

#[test]
fn warning_policy_precedes_env_and_preserves_formal_order() -> Result<(), Box<dyn std::error::Error>>
{
    for warn in [-1, 0, 1, 2] {
        let mut frontend = Frontend::default();
        frontend.eval_source(&format!("options(warn={warn},warnPartialMatchArgs=TRUE)"))?;
        let result = frontend.eval_source("substitute(en={cat('ENV');list(x=3L)},ex=x)");
        let output = frontend.runtime_mut().take_output();
        if warn == 2 {
            assert_eq!(
                result,
                Err(EvalError::Domain(
                    "(converted from warning) partial argument match of 'ex' to 'expr'".into()
                ))
            );
            assert!(output.is_empty());
        } else {
            assert_eq!(result?, vec![Sexp::Integer(vec![Some(3)])]);
            let mut expected = vec![];
            if warn == 1 {
                expected.push(OutputEvent::Stderr(
                    "Warning in substitute(en = { : partial argument match of 'ex' to 'expr'\n"
                        .into(),
                ));
                expected.push(OutputEvent::Stderr(
                    "Warning in substitute(en = { : partial argument match of 'en' to 'env'\n"
                        .into(),
                ));
            }
            expected.push(OutputEvent::Stdout("ENV".into()));
            if warn == 0 {
                expected.push(OutputEvent::Stderr("Warning messages:\n1: In substitute(en = { : partial argument match of 'ex' to 'expr'\n2: In substitute(en = { : partial argument match of 'en' to 'env'\n".into()));
            }
            assert_eq!(output, expected);
        }
    }
    Ok(())
}

fn fail_output(_: OutputEvent) -> std::io::Result<()> {
    Err(std::io::Error::other("substitute host sink"))
}

#[test]
fn env_host_error_is_not_a_substitution_condition() -> Result<(), Box<dyn std::error::Error>> {
    let mut frontend = Frontend::default();
    frontend.runtime_mut().set_output_sink(fail_output);
    assert_eq!(
        frontend.eval_source("substitute(stop('BAD'),{cat('ENV');NULL})"),
        Err(EvalError::Output("substitute host sink".into()))
    );
    Ok(())
}
