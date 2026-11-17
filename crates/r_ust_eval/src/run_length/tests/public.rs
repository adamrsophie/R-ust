use super::*;

#[test]
fn public_run_length_wrapper_creates_class_and_exact_named_boundaries() -> Result<(), EvalError> {
    let mut f = Frontend::default();
    let result=f.eval_source("x<-c(a=1L,b=1L,c=2L);r<-rle(x);identical(class(r),'rle') && identical(r$lengths,structure(c(2L,1L),names=c('c',''))) && identical(r$values,c(b=1L,c=2L)) && identical(inverse.rle(r),c(1L,1L,2L))")?;
    assert_eq!(result.last(), Some(&Sexp::Logical(vec![Some(true)])));
    Ok(())
}

#[test]
fn public_managed_bindings_encode_and_decode_without_losing_storage() -> Result<(), EvalError> {
    let mut f = Frontend::default();
    let owner = f.runtime().global.clone();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(1), Some(2)]),
        attributes: vec![],
    };
    owner.insert(
        "x".into(),
        &ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.transport()?,
    )?;
    let result=f.eval_source("r<-rle(x);identical(r$lengths,c(2L,1L)) && identical(r$values,c(1L,2L)) && identical(inverse.rle(r),c(1L,1L,2L))")?;
    assert_eq!(result.last(), Some(&Sexp::Logical(vec![Some(true)])));
    Ok(())
}

#[test]
fn public_inverse_wrapper_ignores_unforced_dots_and_retains_warning_errors() -> Result<(), EvalError>
{
    let mut f = Frontend::default();
    assert_eq!(
        f.eval_source("inverse.rle(list(lengths=2L,values=7L),stop('never'))")?
            .last(),
        Some(&Sexp::Integer(vec![Some(7), Some(7)]))
    );
    let error = f
        .eval_source("inverse.rle(list(lengths=c('bad','1'),values=c('a','b')))")
        .expect_err("invalid counts");
    let EvalError::WithWarnings { error, warnings } = error else {
        panic!("deferred coercion warning")
    };
    assert!(matches!(*error,EvalError::Domain(ref message) if message=="invalid 'times' value"));
    assert_eq!(warnings.matches("NAs introduced by coercion").count(), 1);
    Ok(())
}
