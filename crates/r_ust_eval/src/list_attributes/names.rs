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
    let view = value.ordinary_list().ok_or(EvalError::NotImplemented(
        "names replacement on this object",
    ))?;
    let elements = view.elements;
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
    let mut elements = elements.to_vec();
    for (index, element) in elements.iter_mut().enumerate() {
        element.name = if matches!(replacement, Sexp::Nil) {
            None
        } else {
            names.get(index).cloned().flatten()
        };
    }
    let names_present = !matches!(replacement, Sexp::Nil);
    let class = view.class.map(|class| {
        (
            class.clone(),
            if !view.names_present && names_present {
                true
            } else {
                view.class_first
            },
        )
    });
    Ok(Sexp::list_with_metadata(elements, names_present, class))
}
