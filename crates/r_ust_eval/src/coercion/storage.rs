use std::borrow::Cow;

use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::AggregateVector;

// Coercion reads storage, not attributes; aggregate children remain rooted transports.
pub(crate) fn read(value: &Sexp) -> Result<Cow<'_, Sexp>, EvalError> {
    match value {
        Sexp::Managed(value) => {
            let borrowed = value.borrow()?;
            Ok(Cow::Owned(read(borrowed.value())?.into_owned()))
        }
        Sexp::AttributedAtomic { values, .. } => Ok(Cow::Owned(values.clone().into_value())),
        Sexp::AttributedAggregate { values, .. } => Ok(Cow::Owned(match values {
            AggregateVector::Vector(values) => Sexp::Vector(values.clone()),
            AggregateVector::Expression(values) => Sexp::Expression {
                elements: values
                    .iter()
                    .cloned()
                    .map(|value| ListElement { name: None, value })
                    .collect(),
                names_present: false,
            },
        })),
        Sexp::Matrix { data, .. } => read(data),
        _ => crate::ordinary_values::read(value),
    }
}
