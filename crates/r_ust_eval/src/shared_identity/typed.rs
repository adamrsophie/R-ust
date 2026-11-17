use super::{Comparison, EvalError, ListElement, Sexp};
use r_ust_sexp::AggregateVector;

pub(super) fn object_flag(value: &Sexp) -> Result<bool, EvalError> {
    let class = match value {
        Sexp::Managed(value) => return Ok(value.view()?.header.is_object()),
        Sexp::Environment(value) => return Ok(value.is_object()?),
        Sexp::ListObject(value) => return object_flag(&value.value()?),
        Sexp::Factor { .. } | Sexp::DataFrame { .. } => return Ok(true),
        Sexp::ClassedList { class, .. }
        | Sexp::NamedList {
            class: Some(class), ..
        } => Some(class.as_ref()),
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => attributes
            .iter()
            .find(|attr| attr.name.as_deref() == Some("class"))
            .map(|attr| &attr.value),
        _ => None,
    };
    let Some(class) = class else { return Ok(false) };
    let class = crate::ordinary_values::read(class)?;
    let values = class
        .character_values()
        .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
    Ok(!values.is_empty())
}

impl Comparison {
    pub(super) fn typed(&mut self, left: &Sexp, right: &Sexp) -> Result<bool, EvalError> {
        let Some(left) = left.attribute_description()? else {
            return Ok(false);
        };
        let Some(right) = right.attribute_description()? else {
            return Ok(false);
        };
        if left.sexptype() != right.sexptype() {
            return Ok(false);
        }
        let Some((left_value, left_attrs)) = parts(&left) else {
            return Ok(false);
        };
        let Some((right_value, right_attrs)) = parts(&right) else {
            return Ok(false);
        };
        Ok(self.compare(&left_value, &right_value)? && self.attributes(left_attrs, right_attrs)?)
    }
}

fn parts(value: &Sexp) -> Option<(Sexp, &[ListElement])> {
    match value {
        Sexp::AttributedAtomic { values, attributes } => {
            Some((values.clone().into_value(), attributes))
        }
        Sexp::AttributedCharacter { values, attributes } => {
            Some((Sexp::String(values.clone()), attributes))
        }
        Sexp::AttributedAggregate { values, attributes } => Some((
            match values {
                AggregateVector::Vector(values) => Sexp::Vector(values.clone()),
                AggregateVector::Expression(values) => Sexp::Expression {
                    elements: values
                        .iter()
                        .cloned()
                        .map(|value| ListElement { name: None, value })
                        .collect(),
                    names_present: false,
                },
            },
            attributes,
        )),
        _ => None,
    }
}

pub(super) fn expanded_attributes(
    attributes: &[ListElement],
) -> Result<Vec<ListElement>, EvalError> {
    attributes
        .iter()
        .map(|attribute| {
            Ok(ListElement {
                name: attribute.name.clone(),
                value: if attribute.name.as_deref() == Some("row.names") {
                    r_ust_sexp::expanded_row_names(&attribute.value)?
                } else {
                    attribute.value.clone()
                },
            })
        })
        .collect()
}
