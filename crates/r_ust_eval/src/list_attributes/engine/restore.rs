use super::*;
mod dataframe;

pub(super) fn value(original: &Sexp, description: Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::Managed(object) = original {
        return Ok(Sexp::Managed(
            object.with_attributes(attributes(&description))?,
        ));
    }
    let result = match description {
        Sexp::AttributedAtomic { values, attributes } if attributes.is_empty() => {
            values.into_value()
        }
        Sexp::AttributedAtomic {
            values: AtomicVector::String(values),
            attributes,
        } if !matches!(original, Sexp::AttributedAtomic { .. }) => {
            Sexp::AttributedCharacter { values, attributes }
        }
        Sexp::AttributedCharacter { values, attributes } if attributes.is_empty() => {
            Sexp::String(values)
        }
        Sexp::AttributedAggregate { values, attributes } => {
            aggregate(original, values, attributes)?
        }
        other => other,
    };
    if let Sexp::ListObject(object) = original {
        if result.ordinary_list().is_some() {
            return Ok(Sexp::ListObject(object.copy_with(&result)?));
        }
    }
    Ok(result)
}

fn aggregate(
    original: &Sexp,
    values: AggregateVector,
    attributes: Vec<ListElement>,
) -> Result<Sexp, EvalError> {
    if let Some(value) = dataframe::restore(original, &values, &attributes)? {
        return Ok(value);
    }
    let plain_names = match find(&attributes, "names") {
        Some(Sexp::Managed(_)) => false,
        Some(names) => stored(names)?.is_empty(),
        None => true,
    };
    let compatible = plain_names
        && attributes
            .iter()
            .all(|a| matches!(a.name.as_deref(), Some("names" | "class")));
    if compatible && !matches!(original, Sexp::AttributedAggregate { .. }) {
        let names_value = find(&attributes, "names").map(payload).transpose()?;
        let names = names_value.as_ref().and_then(Sexp::character_values);
        let present = names.is_some();
        let elements = values
            .values()
            .iter()
            .enumerate()
            .map(|(i, value)| ListElement {
                name: names.and_then(|v| v.get(i)).cloned().flatten(),
                value: value.clone(),
            })
            .collect();
        let class = find(&attributes, "class");
        match &values {
            AggregateVector::Expression(_) if class.is_none() => {
                return Ok(Sexp::Expression {
                    elements,
                    names_present: present,
                })
            }
            AggregateVector::Vector(_)
                if !matches!(original, Sexp::Vector(_)) || !attributes.is_empty() =>
            {
                let class_first = attributes
                    .first()
                    .is_some_and(|a| a.name.as_deref() == Some("class"));
                return Ok(Sexp::list_with_metadata(
                    elements,
                    present,
                    class.map(|c| (c.clone(), class_first)),
                ));
            }
            AggregateVector::Vector(values) => return Ok(Sexp::Vector(values.clone())),
            _ => {}
        }
    }
    Ok(Sexp::AttributedAggregate { values, attributes })
}
