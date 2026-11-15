use crate::{ordinary_values, sequence_elements, EvalError, EvaluatedArgument, ListElement, Sexp};

pub(super) fn concatenate(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let use_names = crate::named_bool_arg(args, &["use.names"], true)?;
    if crate::named_bool_arg(args, &["recursive"], false)? {
        return Err(EvalError::NotImplemented(
            "recursive concatenation with list names",
        ));
    }
    let mut output = Vec::new();
    let mut names_present = false;
    for arg in args
        .iter()
        .filter(|arg| !matches!(arg.name.as_deref(), Some("use.names" | "recursive")))
    {
        let projection = ordinary_values::read(&arg.value)?;
        let (elements, present) = elements(projection.as_ref())?;
        let outer = arg.name.as_deref().filter(|name| !name.is_empty());
        names_present |= present || outer.is_some();
        let count = elements.len();
        for (index, mut element) in elements.into_iter().enumerate() {
            if let Some(outer) = outer {
                let inner = if present {
                    element.name.as_deref().unwrap_or("NA")
                } else {
                    ""
                };
                element.name = Some(if !inner.is_empty() {
                    format!("{outer}.{inner}")
                } else if count == 1 {
                    outer.to_owned()
                } else {
                    format!("{outer}{}", index + 1)
                });
            } else if !present {
                element.name = Some(String::new());
            }
            output.push(element);
        }
    }
    let present = use_names && names_present && !output.is_empty();
    Ok(Sexp::list_with_metadata(output, present, None))
}

fn elements(value: &Sexp) -> Result<(Vec<ListElement>, bool), EvalError> {
    if let Some(view) = value.ordinary_list() {
        return Ok((view.elements.to_vec(), view.names_present));
    }
    if let Sexp::PairList(values) = value {
        let present = values
            .as_slice()
            .iter()
            .any(|element| element.name.is_some());
        let mut elements = values.as_slice().to_vec();
        if present {
            for element in &mut elements {
                if element.name.is_none() {
                    element.name = Some(String::new());
                }
            }
        }
        return Ok((elements, present));
    }
    Ok((
        sequence_elements(value)?
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
        false,
    ))
}

pub(super) fn append(x: &Sexp, values: &Sexp, after: usize) -> Result<Sexp, EvalError> {
    let x = ordinary_values::read(x)?;
    let values = ordinary_values::read(values)?;
    let (mut left, left_names) = elements(x.as_ref())?;
    let (mut right, right_names) = elements(values.as_ref())?;
    let names_present = (left_names || right_names) && !(left.is_empty() && right.is_empty());
    if names_present {
        for (elements, present) in [(&mut left, left_names), (&mut right, right_names)] {
            if !present {
                for element in elements {
                    element.name = Some(String::new());
                }
            }
        }
    }
    left.splice(after..after, right);
    Ok(Sexp::list_with_metadata(left, names_present, None))
}
