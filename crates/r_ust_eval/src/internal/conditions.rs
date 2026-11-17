use super::as_logical;
use crate::{class_value, EvalError, EvaluatedArgument, Sexp};

pub(super) fn inherits(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let what = crate::coercion::storage::read(&args[1].value)?;
    let Some(what) = what.character_values() else {
        return Err(EvalError::Domain(
            "'what' must be a character vector or an object with a nameOfClass() method".into(),
        ));
    };
    let which = crate::coercion::storage::read(&args[2].value)?;
    let Sexp::Logical(which) = which.as_ref() else {
        return Err(EvalError::Domain(
            "'which' must be a length 1 logical vector".into(),
        ));
    };
    let [which] = which.as_slice() else {
        return Err(EvalError::Domain(
            "'which' must be a length 1 logical vector".into(),
        ));
    };
    let which = which.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
    let classes = class_value(&args[0].value)?;
    let classes = crate::coercion::storage::read(&classes)?;
    let Some(classes) = classes.character_values() else {
        return Err(EvalError::Type("invalid class representation".into()));
    };
    let positions = what
        .iter()
        .map(|name| {
            let position = classes
                .iter()
                .position(|class| class == name)
                .map_or(0, |index| index + 1);
            i32::try_from(position)
                .map(Some)
                .map_err(|_| EvalError::Domain("class index overflow".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if which {
        Sexp::Integer(positions)
    } else {
        Sexp::Logical(vec![Some(
            positions.iter().any(|position| *position != Some(0)),
        )])
    })
}

#[cfg(test)]
mod tests;

pub(super) fn gettext(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let string = crate::coercion::storage::read(&args[1].value)?;
    if crate::value_len(string.as_ref())? == 0 {
        return Ok(args[1].value.clone());
    }
    if string.character_values().is_none() {
        return Err(EvalError::Domain("invalid 'string' value".into()));
    }
    let domain = crate::coercion::storage::read(&args[0].value)?;
    let translate = match domain.as_ref() {
        Sexp::Nil => true,
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => match values.first() {
            Some(Some(domain)) => !domain.is_empty(),
            Some(None) => true,
            None => return Err(EvalError::Domain("invalid 'domain' value".into())),
        },
        Sexp::Logical(values) if values.as_slice() == [None] => false,
        _ => return Err(EvalError::Domain("invalid 'domain' value".into())),
    };
    if let (true, Some(trim)) = (translate, args.get(2)) {
        let trim = crate::coercion::storage::read(&trim.value)?;
        let trim = trim.as_ref();
        if matches!(
            trim,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::Matrix { .. }
                | Sexp::Factor { .. }
        ) && crate::value_len(trim)? > 1
        {
            return Err(EvalError::Domain(format!(
                "'length = {}' in coercion to 'logical(1)'",
                crate::value_len(trim)?
            )));
        }
        as_logical(trim).ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))?;
    }
    // shortcut: untranslated C-locale text only, add catalogues with locale support.
    Ok(args[1].value.clone())
}
