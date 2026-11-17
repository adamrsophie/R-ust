use super::{LanguagePrintError, Writer};
use crate::{EvalError, ListElement, Sexp};

impl Writer<'_> {
    pub(super) fn write_graph(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        if self.ancestry.iter().any(|prior| match (prior, value) {
            (Sexp::Managed(a), Sexp::Managed(b)) => a.same_object(b),
            (Sexp::ListObject(a), Sexp::ListObject(b)) => a.same_object(b),
            (Sexp::PairListObject(a), Sexp::PairListObject(b)) => a.same_object(b),
            _ => false,
        }) {
            return Err(EvalError::NotImplemented("deparse cyclic graph value").into());
        }
        self.ancestry.push(value.clone());
        let result = match value {
            Sexp::Managed(owner) => match owner.borrow() {
                Ok(borrow) => self.write(borrow.value()),
                Err(error) => Err(EvalError::from(error).into()),
            },
            _ => match crate::ordinary_values::read(value) {
                Ok(projection) => self.write(projection.as_ref()),
                Err(error) => Err(error.into()),
            },
        };
        self.ancestry.pop();
        result
    }

    pub(super) fn deparse_typed(&mut self, value: &Sexp) -> Result<(), LanguagePrintError> {
        match value {
            Sexp::AttributedAtomic { values, attributes } => {
                let attributes = projected_names(attributes)?;
                self.deparse_atomic(&values.clone().into_value(), &attributes)
            }
            Sexp::AttributedAggregate { values, attributes } => {
                let attributes = projected_names(attributes)?;
                let names = attributes
                    .iter()
                    .find(|a| a.name.as_deref() == Some("names"))
                    .and_then(|a| a.value.character_values());
                let elements: Vec<_> = values
                    .values()
                    .iter()
                    .enumerate()
                    .map(|(index, value)| ListElement {
                        name: names.and_then(|n| n.get(index)).cloned().flatten(),
                        value: value.clone(),
                    })
                    .collect();
                self.deparse_list(
                    &elements,
                    &attributes,
                    matches!(values, r_ust_sexp::AggregateVector::Expression(_)),
                )
            }
            _ => Err(LanguagePrintError::MalformedCall),
        }
    }
}

pub(super) fn projected_names(
    attributes: &[ListElement],
) -> Result<Vec<ListElement>, LanguagePrintError> {
    attributes
        .iter()
        .map(|attribute| {
            let mut result = attribute.clone();
            if attribute.name.as_deref() == Some("names") {
                if let Sexp::Managed(owner) = &attribute.value {
                    let borrow = owner.borrow().map_err(EvalError::from)?;
                    result.value = borrow.value().clone();
                }
            }
            Ok(result)
        })
        .collect()
}

pub(super) fn inspect(value: &Sexp) -> Result<std::borrow::Cow<'_, Sexp>, LanguagePrintError> {
    match value {
        Sexp::Managed(owner) => {
            let borrowed = owner.borrow().map_err(EvalError::from)?;
            Ok(std::borrow::Cow::Owned(
                crate::ordinary_values::read(borrowed.value())?.into_owned(),
            ))
        }
        _ => crate::ordinary_values::read(value).map_err(LanguagePrintError::from),
    }
}
