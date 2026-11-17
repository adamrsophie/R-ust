use crate::{EvalError, Frontend, Sexp};

#[test]
fn public_atomic_c_retains_names_and_default_attribute_policy() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    for source in [
        "identical(c(a=1L,b=1L,c=2L),structure(c(1L,1L,2L),names=c('a','b','c')))",
        "identical(c(outer=structure(1:3,names=c(NA_character_,'','x'))),structure(1:3,names=c('outer.NA','outer2','outer.x')))",
        "identical(c(structure(1:2,names=c('a','b'),note=7L)),structure(1:2,names=c('a','b')))",
    ] {
        assert_eq!(frontend.eval_source(source)?.last(), Some(&Sexp::Logical(vec![Some(true)])), "{source}");
    }
    Ok(())
}

#[test]
fn public_atomic_c_controls_and_empty_storage_match_gnu() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    for source in [
        "identical(c(a=1L,u=FALSE),structure(c(1L,0L),names=c('a','u')))",
        "identical(c(a=1L,use.names=FALSE),1L)",
        "identical(c(a=1L,use.names=NA),structure(1L,names='a'))",
        "identical(c(integer(0),character(0)),character(0))",
        "identical(c(z=NULL,1L),structure(1L,names=''))",
        "identical(c(1L,z=NULL),1L)",
    ] {
        assert_eq!(
            frontend.eval_source(source)?.last(),
            Some(&Sexp::Logical(vec![Some(true)])),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn public_named_c_unblocks_run_length_boundary_names() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let result = frontend.eval_source("x<-c(a=1L,b=1L,c=2L);r<-rle(x);identical(class(r),'rle') && identical(r$lengths,structure(c(2L,1L),names=c('c',''))) && identical(r$values,structure(c(1L,2L),names=c('b','c'))) && identical(inverse.rle(r),c(1L,1L,2L))")?;
    assert_eq!(result.last(), Some(&Sexp::Logical(vec![Some(true)])));
    Ok(())
}
