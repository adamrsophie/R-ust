use super::*;

pub(super) fn unique_value(value: &Sexp) -> Result<Sexp, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let mut out: Vec<ListElement> = Vec::new();
            for element in values {
                if !has_identical(out.iter().map(|slot| &slot.value), &element.value)? {
                    out.push(element.clone());
                }
            }
            Ok(Sexp::List(out))
        }
        Sexp::Vector(values) => {
            let mut out: Vec<Sexp> = Vec::new();
            for value in values {
                if !has_identical(out.iter(), value)? {
                    out.push(value.clone());
                }
            }
            Ok(Sexp::Vector(out))
        }
        _ => {
            let keys = match_keys(value)?;
            let mut seen = Vec::new();
            let positions = keys.into_iter().enumerate().filter_map(|(index, key)| {
                if seen.contains(&key) {
                    None
                } else {
                    seen.push(key);
                    Some(index + 1)
                }
            });
            subset_positions_1_based(value, positions)
        }
    }
}

pub(super) fn duplicated_value(value: &Sexp) -> Result<Sexp, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Sexp::Logical(Vec::new())),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let mut seen: Vec<Sexp> = Vec::new();
            Ok(Sexp::Logical(
                values
                    .iter()
                    .map(|element| {
                        let duplicated = has_identical(seen.iter(), &element.value)?;
                        if !duplicated {
                            seen.push(element.value.clone());
                        }
                        Ok(Some(duplicated))
                    })
                    .collect::<Result<Vec<_>, EvalError>>()?,
            ))
        }
        Sexp::Vector(values) => {
            let mut seen: Vec<Sexp> = Vec::new();
            Ok(Sexp::Logical(
                values
                    .iter()
                    .map(|value| {
                        let duplicated = has_identical(seen.iter(), value)?;
                        if !duplicated {
                            seen.push(value.clone());
                        }
                        Ok(Some(duplicated))
                    })
                    .collect::<Result<Vec<_>, EvalError>>()?,
            ))
        }
        _ => {
            let mut seen = Vec::new();
            Ok(Sexp::Logical(
                match_keys(value)?
                    .into_iter()
                    .map(|key| {
                        let duplicated = seen.contains(&key);
                        if !duplicated {
                            seen.push(key);
                        }
                        Some(duplicated)
                    })
                    .collect(),
            ))
        }
    }
}

pub(super) fn is_list_set_value(value: &Sexp) -> bool {
    matches!(
        value,
        Sexp::ClassedList { .. }
            | Sexp::List(_)
            | Sexp::ListObject(_)
            | Sexp::Vector(_)
            | Sexp::Dots(_)
    )
}

pub(super) fn set_list_values(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(values.iter().map(|element| element.value.clone()).collect()),
        Sexp::Vector(values) => Ok(values.clone()),
        _ => sequence_elements(value),
    }
}

pub(super) fn has_identical<'a>(
    values: impl IntoIterator<Item = &'a Sexp>,
    target: &Sexp,
) -> Result<bool, EvalError> {
    for value in values {
        if sexp_identical(value, target)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn set_list_operation(
    x: &Sexp,
    y: &Sexp,
    operation: SetOperation,
) -> Result<Sexp, EvalError> {
    let x_values = set_list_values(x)?;
    let y_values = set_list_values(y)?;
    let mut seen = Vec::new();
    let mut out = Vec::new();

    match operation {
        SetOperation::Union => {
            for value in x_values.into_iter().chain(y_values) {
                if !has_identical(seen.iter(), &value)? {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
        SetOperation::Intersect => {
            for value in x_values {
                if has_identical(y_values.iter(), &value)? && !has_identical(seen.iter(), &value)? {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
        SetOperation::SetDiff => {
            for value in x_values {
                if !has_identical(y_values.iter(), &value)? && !has_identical(seen.iter(), &value)?
                {
                    seen.push(value.clone());
                    out.push(ListElement { name: None, value });
                }
            }
        }
    }

    Ok(Sexp::List(out))
}

pub(super) fn set_list_equal(x: &Sexp, y: &Sexp) -> Result<bool, EvalError> {
    let x_values = set_list_values(x)?;
    let y_values = set_list_values(y)?;
    for value in &x_values {
        if !has_identical(y_values.iter(), value)? {
            return Ok(false);
        }
    }
    for value in &y_values {
        if !has_identical(x_values.iter(), value)? {
            return Ok(false);
        }
    }
    Ok(true)
}
