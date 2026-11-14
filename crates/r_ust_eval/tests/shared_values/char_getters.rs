use super::*;

#[test]
fn char_scalar_logical_coercion_reaches_encode_string() -> Result {
    for (text, expected) in [
        (Some("TRUE"), Sexp::String(vec![Some("<NA>".into())])),
        (Some("FALSE"), Sexp::String(vec![None])),
    ] {
        let mut frontend = Frontend::default();
        bind(
            frontend.runtime_mut(),
            char_expression(Sexp::Char(text.map(str::to_owned))),
        )?;
        let error = frontend
            .eval_source("as.logical(x)")
            .expect_err("public vector coercion");
        assert!(error
            .to_string()
            .ends_with("cannot coerce type 'char' to vector of type 'logical'"));
        assert_eq!(
            frontend.eval_source("encodeString(NA_character_, na.encode=x)")?,
            vec![expected]
        );
    }
    for text in [None, Some("unrecognized")] {
        let mut frontend = Frontend::default();
        bind(
            frontend.runtime_mut(),
            char_expression(Sexp::Char(text.map(str::to_owned))),
        )?;
        let error = frontend
            .eval_source("encodeString(NA_character_, na.encode=x)")
            .expect_err("invalid scalar flag");
        assert!(error.to_string().ends_with("invalid 'na.encode' value"));
    }
    Ok(())
}

#[test]
fn char_data_getters_differ_from_raw_ast_evaluation() -> Result {
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        char_expression(Sexp::Char(Some("error".into()))),
    )?;
    assert_eq!(
        frontend.eval_source("mode(x);names(x);attributes(x);is.atomic(x);is.character(x)")?,
        vec![
            Sexp::String(vec![Some("char".into())]),
            Sexp::Nil,
            Sexp::Nil,
            Sexp::Logical(vec![Some(true)]),
            Sexp::Logical(vec![Some(false)])
        ]
    );
    for source in ["class(x)", "print(x)"] {
        let error = frontend.eval_source(source).expect_err(source);
        assert!(
            error
                .to_string()
                .ends_with("cannot have attributes on a CHARSXP"),
            "{source}: {error}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
    assert_eq!(
        frontend.eval_source("is.na(x)")?,
        vec![Sexp::Logical(vec![Some(false); 5])]
    );
    let events = frontend.runtime_mut().take_output();
    assert_eq!(events.len(), 1);
    let r_ust_eval::OutputEvent::Stderr(warning) = &events[0] else {
        panic!("warning event")
    };
    assert!(warning.contains("is.na() applied to non-(list or vector) of type 'char'"));
    assert_eq!(
        frontend
            .runtime_mut()
            .eval(&Sexp::Char(Some("error".into()))),
        Err(EvalError::Type(
            "unimplemented type 'char' in 'eval'\n".into()
        ))
    );
    Ok(())
}

#[test]
fn char_public_attribute_boundaries_match_r() -> Result {
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        char_expression(Sexp::Char(Some("error".into()))),
    )?;
    for source in [
        "attr(x,'custom')",
        "attr(x,'class')",
        "attr(x,'na',exact=TRUE)",
        "attr(x,'',exact=TRUE)",
    ] {
        assert_eq!(frontend.eval_source(source)?, vec![Sexp::Nil], "{source}");
    }
    for (source, expected) in [
        ("attr(x,'names')", "cannot have attributes on a CHARSXP"),
        ("attr(x,'na')", "cannot have attributes on a CHARSXP"),
        ("attr(x,'')", "cannot have attributes on a CHARSXP"),
        (
            "attr(x,'custom')<-1L",
            "cannot set an attribute on a CHARSXP",
        ),
        (
            "attr(x,'custom')<-NULL",
            "cannot set an attribute on a CHARSXP",
        ),
        ("class(x)<-'box'", "cannot set an attribute on a CHARSXP"),
        ("class(x)<-NULL", "cannot set an attribute on a CHARSXP"),
        ("names(x)<-'a'", "names() applied to a non-vector"),
        ("names(x)<-NULL", "cannot have attributes on a CHARSXP"),
        (
            "attributes(x)<-list(custom=1L)",
            "cannot set an attribute on a CHARSXP",
        ),
    ] {
        let error = frontend.eval_source(source).expect_err(source);
        assert!(error.to_string().ends_with(expected), "{source}: {error}");
        assert_eq!(
            frontend.runtime().get("x")?,
            Some(Sexp::Char(Some("error".into())))
        );
    }
    for source in ["attributes(x)<-NULL", "attributes(x)<-list()"] {
        frontend.eval_source(source)?;
        assert_eq!(
            frontend.runtime().get("x")?,
            Some(Sexp::Char(Some("error".into())))
        );
    }
    assert!(frontend.runtime_mut().take_output().is_empty());
    Ok(())
}
