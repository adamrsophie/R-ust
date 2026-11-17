use super::*;

pub(super) fn list(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    let value = crate::ordinary_values::read(value)?.into_owned();
    if let Some(view) = value.ordinary_list() {
        return Ok(view.elements.to_vec());
    }
    match value {
        Sexp::Nil => Ok(vec![]),
        Sexp::Vector(values) => Ok(values
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect()),
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            attributes,
        } => {
            let names = find(&attributes, "names").map(payload).transpose()?;
            let names = names.as_ref().and_then(Sexp::character_values);
            Ok(values
                .into_iter()
                .enumerate()
                .map(|(i, value)| ListElement {
                    name: names.and_then(|v| v.get(i)).cloned().flatten(),
                    value,
                })
                .collect())
        }
        _ => Err(EvalError::Type("attributes must be a list or NULL".into())),
    }
}
pub(super) fn attribute_list(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    let attrs = list(value)?;
    let named = crate::names_value(value)?;
    if !attrs.is_empty() && matches!(named, Sexp::Nil) {
        return Err(EvalError::Type("attributes must be named".into()));
    }
    for (i, attr) in attrs.iter().enumerate().skip(1) {
        if attr.name.as_deref() == Some("") {
            return Err(EvalError::Type(format!(
                "all attributes must have names [{} does not]",
                i + 1
            )));
        }
    }
    Ok(attrs)
}
pub(super) fn ordered(attrs: &[ListElement]) -> Vec<&ListElement> {
    let dim = attrs.iter().position(|a| a.name.as_deref() == Some("dim"));
    dim.into_iter()
        .map(|i| &attrs[i])
        .chain(
            attrs
                .iter()
                .enumerate()
                .filter(|(i, _)| Some(*i) != dim)
                .map(|(_, a)| a),
        )
        .collect()
}

pub(super) fn resized_attributes(
    value: &Sexp,
    sizes: (usize, usize),
) -> Result<Vec<ListElement>, EvalError> {
    let attributes = stored(value)?;
    if sizes.0 == sizes.1 {
        return Ok(attributes);
    }
    let Some(names) = find(&attributes, "names") else {
        return Ok(Vec::new());
    };
    let names = payload(names)?;
    let mut names = names
        .character_values()
        .ok_or_else(|| EvalError::Type("invalid names on resized vector".into()))?
        .to_vec();
    names.resize(sizes.1, Some(String::new()));
    Ok(vec![ListElement {
        name: Some("names".into()),
        value: Sexp::String(names),
    }])
}
