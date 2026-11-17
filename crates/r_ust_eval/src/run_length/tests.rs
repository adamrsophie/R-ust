use super::storage;
use crate::{EvalError, EvaluatedArgument, Frontend, ListElement, RuntimeState, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector, Environment, ValueImport, ValueObject};
mod atomic;
mod boundaries;
mod identity;
mod inverse;
mod metadata;
mod public;
mod records;

fn variants(raw: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    let owner = Environment::global();
    let described = raw.attribute_description()?.unwrap_or_else(|| raw.clone());
    Ok(vec![
        raw.clone(),
        described.clone(),
        ValueObject::import(&owner, raw, ValueImport::Evaluated)?.transport()?,
        ValueObject::import(&owner, &described, ValueImport::Evaluated)?.transport()?,
    ])
}

fn encoded(value: Sexp) -> Result<Sexp, EvalError> {
    super::encode(&[EvaluatedArgument { name: None, value }])
}

fn decoded(lengths: Sexp, values: Sexp) -> Result<Sexp, EvalError> {
    let value = Sexp::List(vec![
        ListElement {
            name: Some("lengths".into()),
            value: lengths,
        },
        ListElement {
            name: Some("values".into()),
            value: values,
        },
    ]);
    RuntimeState::default().inverse_rle(&[EvaluatedArgument { name: None, value }])
}
