use crate::{EvalError, Sexp};
use r_ust_sexp::{Environment, ValueImport, ValueObject};
mod atomic;
mod bindings;
mod expressions;
mod lengths;
mod storage;

fn managed(owner: &Environment, value: &Sexp) -> Result<Sexp, EvalError> {
    Ok(ValueObject::import(owner, value, ValueImport::Evaluated)?.transport()?)
}
