mod named_assignment;
pub(super) use named_assignment::assign_list_name;

use super::{
    assign_list_position, extraction_position, recycle_data_frame_column, string_scalar,
    subassign_value, value_len, EvalError, ListElement, Sexp,
};

pub(super) fn extract_value(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Sexp::PairList(values) = value {
        return extract_value(&Sexp::List(values.as_slice().to_vec()), index);
    }
    if let Ok(name) = string_scalar(index, "extraction index") {
        return match value {
            Sexp::DataFrame { columns, .. } => Ok(columns
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .map(|element| element.value.clone())
                .unwrap_or(Sexp::Nil)),
            Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::List(values) => Ok(values
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .map(|element| element.value.clone())
                .unwrap_or(Sexp::Nil)),
            other => Err(EvalError::Type(format!(
                "cannot extract named element from type {:?}",
                other.sexptype()
            ))),
        };
    }

    let position = extraction_position(index)?;
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::DataFrame { columns, .. } => Ok(columns
            .get(position)
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        Sexp::Logical(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Logical(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Integer(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Integer(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Real(values) => Ok(values
            .get(position)
            .copied()
            .map(|value| Sexp::Real(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => Ok(values
            .get(position)
            .cloned()
            .map(|value| Sexp::String(vec![value]))
            .unwrap_or(Sexp::Nil)),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(codes
            .get(position)
            .map(|code| Sexp::Factor {
                codes: vec![*code],
                levels: levels.clone(),
                ordered: *ordered,
            })
            .unwrap_or(Sexp::Nil)),
        Sexp::Vector(values) => Ok(values.get(position).cloned().unwrap_or(Sexp::Nil)),
        Sexp::Matrix { data, .. } => extract_value(data, index),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => Ok(values
            .get(position)
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        other => Err(EvalError::Type(format!(
            "cannot extract from value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn dollar_value(value: &Sexp, name: &str) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Sexp::PairList(values) = value {
        return dollar_value(&Sexp::List(values.as_slice().to_vec()), name);
    }
    match value {
        Sexp::DataFrame { columns, .. } => Ok(columns
            .iter()
            .find(|element| element.name.as_deref() == Some(name))
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => Ok(values
            .iter()
            .find(|element| element.name.as_deref() == Some(name))
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        other => Err(EvalError::Type(format!(
            "cannot use $ on value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn extract_assign_value(
    value: &Sexp,
    index: &Sexp,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        let projection = crate::ordinary_values::read(value)?;
        let result = extract_assign_value(projection.as_ref(), index, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    if let Sexp::PairList(values) = value {
        if let Ok(name) = string_scalar(index, "extraction index") {
            return crate::pairlist_values::assign_name(values.as_slice(), &name, replacement);
        }
        let result =
            extract_assign_value(&Sexp::List(values.as_slice().to_vec()), index, replacement)?;
        return crate::pairlist_values::restore(result);
    }
    if let Ok(name) = string_scalar(index, "extraction index") {
        return assign_list_name(value, &name, replacement);
    }

    match value {
        Sexp::DataFrame { columns, nrow } => {
            let mut out = columns.clone();
            let position = extraction_position(index)?;
            if position > out.len() {
                return Err(EvalError::Domain(
                    "data frame column assignment index out of bounds".to_string(),
                ));
            }
            let new_nrow = if *nrow == 0 {
                value_len(replacement)?
            } else {
                *nrow
            };
            let value = recycle_data_frame_column(replacement, new_nrow)?;
            if position == out.len() {
                out.push(ListElement {
                    name: Some(format!("X{}", position + 1)),
                    value,
                });
            } else {
                out[position].value = value;
            }
            Ok(Sexp::DataFrame {
                columns: out,
                nrow: new_nrow,
            })
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let mut out = values.clone();
            let position = extraction_position(index)?;
            if matches!(replacement, Sexp::Nil) {
                if position < out.len() {
                    out.remove(position);
                }
            } else {
                assign_list_position(&mut out, position, replacement);
            }
            let reconstructed = out.len() != values.len();
            let mut result = crate::list_attributes::preserve_class(value, out);
            if reconstructed && values.iter().any(|element| element.name.is_some()) {
                if let Sexp::ClassedList { class_first, .. } = &mut result {
                    *class_first = false;
                }
            }
            Ok(result)
        }
        Sexp::Nil if matches!(replacement, Sexp::Nil) => Ok(Sexp::Nil),
        Sexp::Nil => {
            let mut out = Vec::new();
            assign_list_position(&mut out, extraction_position(index)?, replacement);
            Ok(crate::list_attributes::preserve_class(value, out))
        }
        _ => subassign_value(value, Some(index), replacement),
    }
}

pub(super) fn dollar_assign_value(
    value: &Sexp,
    name: &str,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        let projection = crate::ordinary_values::read(value)?;
        let result = dollar_assign_value(projection.as_ref(), name, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    let mut result = assign_list_name(value, name, replacement)?;
    if let (
        Sexp::ClassedList { elements, .. },
        Sexp::ClassedList {
            elements: updated,
            class_first,
            ..
        },
    ) = (value, &mut result)
    {
        if elements.iter().all(|element| element.name.is_none())
            && updated.iter().any(|element| element.name.is_some())
        {
            *class_first = false;
        }
    }
    Ok(result)
}
