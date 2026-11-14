use super::preserve_class;
use crate::{string_values, EvalError, Sexp};

pub(crate) fn assign_names(value: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::PairListObject(object) = value {
        let result = assign_names(&Sexp::pairlist(object.elements()?), replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    if let Sexp::PairList(values) = value {
        return crate::pairlist_values::assign_names(values.as_slice(), replacement);
    }
    if let Sexp::ListObject(object) = value {
        let result = assign_names(&object.value()?, replacement)?;
        return crate::ordinary_values::replaced(value, result);
    }
    if matches!(value, Sexp::Char(_)) {
        return Err(EvalError::Domain(if matches!(replacement, Sexp::Nil) {
            "cannot have attributes on a CHARSXP".into()
        } else {
            "names() applied to a non-vector".into()
        }));
    }
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    let elements = match value {
        Sexp::List(elements) | Sexp::ClassedList { elements, .. } => elements.as_slice(),
        Sexp::PairList(values) => values.as_slice(),
        _ => {
            return Err(EvalError::NotImplemented(
                "names replacement on this object",
            ))
        }
    };
    let names = if matches!(replacement, Sexp::Nil) {
        Vec::new()
    } else {
        string_values(replacement)?
    };
    if names.len() > elements.len() {
        return Err(EvalError::Domain(format!(
            "'names' attribute [{}] must be the same length as the vector [{}]",
            names.len(),
            elements.len()
        )));
    }
    // shortcut: ListElement cannot distinguish absent from all-missing names; extend names storage before supporting these attributes.
    if !matches!(replacement, Sexp::Nil) && names.iter().all(Option::is_none) {
        return Err(EvalError::NotImplemented(
            "empty or all-missing names attribute",
        ));
    }
    let adding_names = elements.iter().all(|element| element.name.is_none());
    let mut elements = elements.to_vec();
    for (index, element) in elements.iter_mut().enumerate() {
        element.name = if matches!(replacement, Sexp::Nil) {
            None
        } else {
            names.get(index).cloned().flatten()
        };
    }
    if matches!(value, Sexp::PairList(_)) {
        return Ok(Sexp::pairlist(elements));
    }
    let mut result = preserve_class(value, elements);
    if adding_names {
        if let Sexp::ClassedList { class_first, .. } = &mut result {
            *class_first = true;
        }
    }
    Ok(result)
}
