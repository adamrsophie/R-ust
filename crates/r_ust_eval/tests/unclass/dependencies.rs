use super::*;

// These active outcomes require the declared persistent attribute foundation
// and public attribute consumers; no metadata-dropping fallback is accepted.
#[test]
fn unclass_factor_retains_levels_and_integer_storage() -> Result {
    let mut f = Frontend::default();
    bind(
        f.runtime_mut(),
        "x",
        Sexp::Factor {
            codes: vec![Some(2), None, Some(1)],
            levels: vec!["a".into(), "b".into(), "unused".into()],
            ordered: false,
        },
    )?;
    truth(&mut f, "u<-unclass(x);typeof(u)=='integer' && identical(attr(u,'levels'),c('a','b','unused')) && is.null(attr(u,'class')) && is.object(x)")
}
#[test]
fn unclass_dataframe_preserves_names_row_names_and_columns() -> Result {
    let mut f = Frontend::default();
    bind(
        f.runtime_mut(),
        "x",
        Sexp::DataFrame {
            columns: vec![attribute("a", Sexp::Integer(vec![Some(1), Some(2)]))],
            nrow: 2,
        },
    )?;
    truth(&mut f, "u<-unclass(x);is.list(u) && identical(names(u),'a') && identical(attr(u,'row.names'),1:2) && is.null(attr(u,'class')) && is.object(x)")
}
#[test]
fn unclass_classed_character_retains_ordered_other_attributes() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "x<-structure(c('a',NA_character_),before=1L,class='box',after=2L);u<-unclass(x);identical(names(attributes(u)),c('before','after')) && identical(as.character(u),c('a',NA_character_)) && identical(attr(x,'class'),'box')")
}
#[test]
fn unclass_classed_environment_errors_without_touching_plain_environment() -> Result {
    let mut f = Frontend::default();
    f.eval_source("e<-new.env();class(e)<-'box'")?;
    assert!(f
        .eval_source("unclass(e)")
        .expect_err("classed environment rejected")
        .to_string()
        .contains("cannot unclass an environment"));
    truth(&mut f, "identical(unclass(globalenv()),globalenv())")
}
