use std::borrow::Cow;

use crate::{EvalError, Sexp};

pub(super) fn read(value: &Sexp) -> Result<Cow<'_, Sexp>, EvalError> {
    Ok(match value {
        Sexp::ListObject(value) => Cow::Owned(value.value()?),
        Sexp::PairListObject(value) => Cow::Owned(Sexp::pairlist(value.elements()?)),
        _ => Cow::Borrowed(value),
    })
}

pub(super) fn replaced(original: &Sexp, replacement: Sexp) -> Result<Sexp, EvalError> {
    match original {
        Sexp::ListObject(original) => Ok(Sexp::ListObject(original.copy_with(&replacement)?)),
        Sexp::PairListObject(original) => match replacement {
            Sexp::PairList(values) => Ok(original
                .copy_with(values.as_slice())?
                .map(Sexp::PairListObject)
                .unwrap_or(Sexp::Nil)),
            Sexp::Nil => Ok(Sexp::Nil),
            _ => Ok(replacement),
        },
        _ => Ok(replacement),
    }
}
