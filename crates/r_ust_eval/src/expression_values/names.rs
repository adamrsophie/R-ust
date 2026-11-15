use crate::{EvalError, ListElement, Sexp};

pub(crate) fn names(elements: &[ListElement], present: bool) -> Sexp {
    if present {
        Sexp::String(
            elements
                .iter()
                .map(|element| element.name.clone())
                .collect(),
        )
    } else {
        Sexp::Nil
    }
}

pub(crate) fn assign_names(
    elements: &[ListElement],
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    let names_present = !matches!(replacement, Sexp::Nil);
    let names = if names_present {
        crate::string_values(replacement)?
    } else {
        Vec::new()
    };
    if names.len() > elements.len() {
        return Err(EvalError::Domain(format!(
            "'names' attribute [{}] must be the same length as the vector [{}]",
            names.len(),
            elements.len()
        )));
    }
    let elements = elements
        .iter()
        .enumerate()
        .map(|(index, element)| ListElement {
            name: names.get(index).cloned().flatten(),
            value: element.value.clone(),
        })
        .collect();
    Ok(Sexp::Expression {
        elements,
        names_present,
    })
}
