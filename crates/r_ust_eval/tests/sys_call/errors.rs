use super::*;

#[test]
fn sys_call_bounds_invalid_values_and_public_argument_errors_keep_source_call() -> Result {
    let mut f = Frontend::default();
    for (expression, message) in [
        ("sys.call(1L)", "not that many frames on the stack"),
        ("sys.call(-1L)", "not that many frames on the stack"),
        ("sys.call(NA_integer_)", "invalid 'which' argument"),
        ("sys.call(integer())", "invalid 'which' argument"),
        ("sys.call(list(0L))", "invalid 'which' argument"),
        ("sys.call(globalenv())", "invalid 'which' argument"),
        ("sys.call(0L,1L)", "unused argument (1)"),
        ("sys.call(other=0L)", "unused argument (other = 0)"),
    ] {
        let program = format!(
            "tryCatch({expression},error=function(e)list(conditionMessage(e),conditionCall(e)))"
        );
        let result = value(&mut f, &program)?;
        let expected_call = value(&mut f, &format!("quote({expression})"))?;
        assert_eq!(
            result,
            list(vec![
                Sexp::String(vec![Some(message.into())]),
                expected_call
            ])
        );
    }
    Ok(())
}

#[test]
fn sys_call_direct_variadic_arity_evaluates_arguments_before_default_minus_one() -> Result {
    let mut f = Frontend::default();
    for source in [".Internal(sys.call())", ".Internal(sys.call(a=0L,b=0L))"] {
        assert_eq!(
            f.eval_source(source),
            Err(EvalError::Domain(
                "not that many frames on the stack".into()
            ))
        );
    }
    assert_eq!(value(&mut f, ".Internal(sys.call(0L))")?, Sexp::Nil);
    assert!(f
        .eval_source(".Internal(sys.call(a={cat('A|');'bad'},b={cat('B|');0L}))")
        .is_err());
    assert_eq!(output(&mut f), "A|B|");
    assert!(f
        .eval_source(".Internal(sys.call(,))")
        .expect_err("empty argument")
        .to_string()
        .contains("argument 1 is empty"));
    Ok(())
}

#[test]
fn sys_call_coercion_warning_and_error_both_name_original_wrapper_call() -> Result {
    let mut f = Frontend::default();
    for (which, warning) in [
        ("'bad'", "NAs introduced by coercion"),
        ("Inf", "NAs introduced by coercion to integer range"),
    ] {
        let program=format!("seen<-NULL;x<-tryCatch(withCallingHandlers(sys.call({which}),warning=function(w){{seen<<-list(conditionMessage(w),conditionCall(w));invokeRestart('muffleWarning')}}),error=function(e)list(conditionMessage(e),conditionCall(e)));list(x,seen)");
        let result = value(&mut f, &program)?;
        let call = value(&mut f, &format!("quote(sys.call({which}))"))?;
        assert_eq!(
            result,
            list(vec![
                list(vec![
                    Sexp::String(vec![Some("invalid 'which' argument".into())]),
                    call.clone()
                ]),
                list(vec![Sexp::String(vec![Some(warning.into())]), call])
            ])
        );
    }
    Ok(())
}
