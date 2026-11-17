use super::*;

#[test]
fn failed_attribute_replacement_is_atomic() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(1:3,note=1L);y<-x;tryCatch(attributes(x)<-list(note=2L,dim=c(2L,2L)),error=function(e)NULL);identical(x,y) && identical(attributes(x),list(note=1L))")?;
    Ok(())
}

#[test]
fn invalid_metadata_errors_preserve_original() -> Result {
    for (setter, message) in [
        (
            "attr(x,'class')<-1L",
            "attempt to set invalid 'class' attribute",
        ),
        (
            "attr(x,'comment')<-7L",
            "attempt to set invalid 'comment' attribute",
        ),
        (
            "attr(x,'dim')<-c(2L,2L)",
            "dims [product 4] do not match the length of object [3]",
        ),
        (
            "attr(x,'dim')<-c(NA_integer_,3L)",
            "the dims contain missing values",
        ),
        (
            "attr(x,'dim')<-c(-1L,3L)",
            "the dims contain negative values",
        ),
        (
            "names(x)<-c('a','b','c','d')",
            "'names' attribute [4] must be the same length as the vector [3]",
        ),
    ] {
        let mut f = Frontend::default();
        f.eval_source("x<-1:3;y<-x")?;
        let error = f.eval_source(setter).expect_err(setter).to_string();
        assert!(error.contains(message), "{setter}: {error}");
        truth(&mut f, "identical(x,y)")?;
    }
    Ok(())
}

#[test]
fn raw_storage_roundtrip_preserves_payload_metadata_and_alias() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(c(0L,1L,255L),note=7L);saved<-x;storage.mode(x)<-'raw';identical(as.integer(x),c(0L,1L,255L)) && identical(as.character(x),c('00','01','ff')) && identical(attributes(x),list(note=7L)) && identical(saved,structure(c(0L,1L,255L),note=7L))")?;
    truth(&mut f, "storage.mode(x)<-'integer';identical(x,saved)")?;
    Ok(())
}

#[test]
fn dim_coercion_warns_before_error_after_rhs_effect() -> Result {
    let mut f = Frontend::default();
    f.eval_source("x<-1:3")?;
    let error = f
        .eval_source("attr(x,{cat('NAME|');'dim'})<-{cat('RHS|');c('bad','3')}")
        .expect_err("invalid dimensions");
    let r_ust_eval::EvalError::WithWarnings { error, warnings } = error else {
        panic!("expected deferred warning on the error");
    };
    assert_eq!(
        *error,
        r_ust_eval::EvalError::Domain("the dims contain missing values".into())
    );
    assert!(warnings.starts_with("Warning message:\n"));
    assert_eq!(warnings.matches("NAs introduced by coercion").count(), 1);
    let events = f.runtime_mut().take_output();
    let output: String = events
        .iter()
        .filter_map(|e| {
            if let r_ust_eval::OutputEvent::Stdout(s) = e {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(output, "RHS|NAME|");
    assert!(events
        .iter()
        .all(|event| matches!(event, r_ust_eval::OutputEvent::Stdout(_))));
    Ok(())
}

#[test]
fn custom_names_coercion_retains_correct_expected_dispatch() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "as.character.custom<-function(x,...)c('A','B','C');x<-1:3;names(x)<-structure(1:3,class='custom');identical(names(x),c('A','B','C'))")
}

#[test]
fn factor_storage_mode_cannot_change() -> Result {
    let mut f = Frontend::default();
    f.eval_source("x<-structure(1:3,levels=c('a','b','c'),class='factor')")?;
    assert!(f
        .eval_source("storage.mode(x)<-'double'")
        .expect_err("factor mode")
        .to_string()
        .contains("invalid to change the storage mode of a factor"));
    truth(&mut f, "typeof(x)=='integer' && is.object(x)")
}

#[test]
fn storage_list_and_raw_retain_correct_expected_roles() -> Result {
    for mode in ["list", "raw"] {
        let mut f = Frontend::default();
        truth(&mut f,&format!("x<-structure(1:3,note=7L);storage.mode(x)<-'{mode}';typeof(x)=='{mode}' && identical(attributes(x),list(note=7L))"))?;
    }
    Ok(())
}
