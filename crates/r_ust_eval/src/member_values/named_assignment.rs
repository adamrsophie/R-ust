use crate::{recycle_data_frame_column, value_len, EvalError, ListElement, Sexp};

pub(crate) fn assign_list_name(
    value: &Sexp,
    name: &str,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        let projection = crate::ordinary_values::read(value)?;
        let result = assign_list_name(projection.as_ref(), name, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    if let Sexp::PairList(values) = value {
        return crate::pairlist_values::assign_name(values.as_slice(), name, replacement);
    }
    if let Sexp::DataFrame { columns, nrow } = value {
        let mut out = columns.clone();
        let new_nrow = if *nrow == 0 {
            value_len(replacement)?
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

    let view = value.ordinary_list();
    let names_present = view.is_some_and(|view| view.names_present);
    let mut out = match value {
        Sexp::Nil => Vec::new(),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::List(values) => values.clone(),
        other => {
            return Err(EvalError::Type(format!(
                "cannot assign named element into value of type {:?}",
                other.sexptype()
            )))
        }
    };

    let original_len = out.len();
    if let Some(index) = out
        .iter()
        .position(|element| element.name.as_deref() == Some(name))
    {
        if matches!(replacement, Sexp::Nil) {
            out.remove(index);
        } else {
            out[index].value = replacement.clone();
        }
    } else if !matches!(replacement, Sexp::Nil) {
        if !names_present {
            for element in &mut out {
                element.name = Some(String::new());
            }
        }
        out.push(ListElement {
            name: Some(name.to_string()),
            value: replacement.clone(),
        });
    }
    let present = names_present || out.iter().any(|element| element.name.is_some());
    let class = view.and_then(|view| {
        view.class.map(|class| {
            (
                class.clone(),
                if out.len() != original_len && names_present {
                    false
                } else {
                    view.class_first
                },
            )
        })
    });
    Ok(Sexp::list_with_metadata(out, present, class))
}
