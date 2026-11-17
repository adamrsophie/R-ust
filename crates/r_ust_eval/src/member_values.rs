mod extraction;
mod language;
pub(super) use extraction::extract_value;
mod named_assignment;
mod typed;
pub(super) use named_assignment::assign_list_name;

use super::{
    assign_list_position, extraction_position, recycle_data_frame_column, string_scalar,
    subassign_value, value_len, EvalError, ListElement, Sexp,
};

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
        | Sexp::NamedList {
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
    if value.sexptype() == r_ust_sexp::SexpType::Language {
        return language::assign(value, index, replacement);
    }
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

    if let Some(view) = value.ordinary_list() {
        let mut out = view.elements.to_vec();
        let position = extraction_position(index)?;
        if matches!(replacement, Sexp::Nil) {
            if position < out.len() {
                out.remove(position);
            }
        } else {
            assign_list_position(&mut out, position, replacement);
            if view.names_present {
                for element in out.iter_mut().skip(view.elements.len()) {
                    element.name = Some(String::new());
                }
            }
        }
        let class = view.class.map(|class| {
            (
                class.clone(),
                if out.len() != view.elements.len() && view.names_present {
                    false
                } else {
                    view.class_first
                },
            )
        });
        return Ok(Sexp::list_with_metadata(out, view.names_present, class));
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
