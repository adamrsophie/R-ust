use super::{
    assign_list_position, extraction_position, recycle_data_frame_column, string_scalar,
    subassign_value, value_len, EvalError, ListElement, Sexp,
};

pub(super) fn extract_value(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    if let Ok(name) = string_scalar(index, "extraction index") {
        return match value {
            Sexp::DataFrame { columns, .. } => Ok(columns
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
                .map(|element| element.value.clone())
                .unwrap_or(Sexp::Nil)),
            Sexp::List(values) => Ok(values
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
        Sexp::String(values) => Ok(values
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
        Sexp::List(values) => Ok(values
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
    match value {
        Sexp::DataFrame { columns, .. } => Ok(columns
            .iter()
            .find(|element| element.name.as_deref() == Some(name))
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil)),
        Sexp::List(values) => Ok(values
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
                value_len(replacement)
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
        Sexp::List(values) => {
            let mut out = values.clone();
            assign_list_position(&mut out, extraction_position(index)?, replacement);
            Ok(Sexp::List(out))
        }
        Sexp::Nil => {
            let mut out = Vec::new();
            assign_list_position(&mut out, extraction_position(index)?, replacement);
            Ok(Sexp::List(out))
        }
        _ => subassign_value(value, Some(index), replacement),
    }
}

pub(super) fn dollar_assign_value(
    value: &Sexp,
    name: &str,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    assign_list_name(value, name, replacement)
}

pub(super) fn assign_list_name(
    value: &Sexp,
    name: &str,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    if let Sexp::DataFrame { columns, nrow } = value {
        let mut out = columns.clone();
        let new_nrow = if *nrow == 0 {
            value_len(replacement)
        } else {
            *nrow
        };
        let value = recycle_data_frame_column(replacement, new_nrow)?;
        if let Some(element) = out
            .iter_mut()
            .find(|element| element.name.as_deref() == Some(name))
        {
            element.value = value;
        } else {
            out.push(ListElement {
                name: Some(name.to_string()),
                value,
            });
        }
        return Ok(Sexp::DataFrame {
            columns: out,
            nrow: new_nrow,
        });
    }

    let mut out = match value {
        Sexp::Nil => Vec::new(),
        Sexp::List(values) => values.clone(),
        other => {
            return Err(EvalError::Type(format!(
                "cannot assign named element into value of type {:?}",
                other.sexptype()
            )))
        }
    };

    if let Some(element) = out
        .iter_mut()
        .find(|element| element.name.as_deref() == Some(name))
    {
        element.value = replacement.clone();
    } else {
        out.push(ListElement {
            name: Some(name.to_string()),
            value: replacement.clone(),
        });
    }
    Ok(Sexp::List(out))
}
