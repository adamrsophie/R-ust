mod assignment;
mod leaf;
mod values;

use super::{matching, NameMatching};
use crate::{EvalError, ListElement, Sexp};
pub(super) use assignment::assign;

pub(super) fn extract(
    value: &Sexp,
    subscript: &Sexp,
    matching: NameMatching,
    warn: &mut impl FnMut(&str, bool) -> Result<(), EvalError>,
) -> Result<Sexp, EvalError> {
    let length = crate::value_len(subscript)?;
    if length == 0 {
        return super::language::index::read(&[], subscript).map(|_| Sexp::Nil);
    }
    let mut value = value.clone();
    for level in 0..length - 1 {
        let (_, child) = descend(&value, subscript, level, matching, warn)?;
        value = child;
    }
    let index = if length == 1 {
        subscript.clone()
    } else {
        at(subscript, length - 1)?
    };
    let elements = values::scalars(&value)?;
    let position = matching::position(&elements, &index, matching, warn)?;
    let Some(position) = position else {
        return if values::slots(&value)?.is_some() {
            Ok(Sexp::Nil)
        } else {
            Err(EvalError::Domain("subscript out of bounds".into()))
        };
    };
    let result = elements
        .get(position)
        .ok_or_else(|| EvalError::Domain("subscript out of bounds".into()))?;
    let storage = crate::coercion::storage::read(&value)?;
    if values::slots(&value)?.is_none()
        && !matches!(
            storage.as_ref(),
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::Raw(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::Factor { .. }
        )
    {
        return Err(EvalError::Type(format!(
            "invalid type/length ({}/1) in vector allocation",
            crate::metadata::typeof_value(&value)
        )));
    }
    Ok(result.value.clone())
}

fn at(index: &Sexp, position: usize) -> Result<Sexp, EvalError> {
    Ok(match index {
        Sexp::Logical(v) => Sexp::Logical(vec![v[position]]),
        Sexp::Integer(v) => Sexp::Integer(vec![v[position]]),
        Sexp::Real(v) => Sexp::Real(vec![v[position]]),
        Sexp::Raw(v) => Sexp::Raw(vec![v[position]]),
        Sexp::String(v) | Sexp::AttributedCharacter { values: v, .. } => {
            Sexp::String(vec![v[position].clone()])
        }
        Sexp::Factor { codes, .. } => Sexp::Integer(vec![codes[position]
            .map(i32::try_from)
            .transpose()
            .map_err(|_| EvalError::Domain("subscript out of bounds".into()))?]),
        _ => {
            return Err(EvalError::Type(format!(
                "invalid subscript type '{}'",
                crate::metadata::typeof_value(index)
            )))
        }
    })
}

fn descend(
    value: &Sexp,
    subscript: &Sexp,
    level: usize,
    matching: NameMatching,
    warn: &mut impl FnMut(&str, bool) -> Result<(), EvalError>,
) -> Result<(usize, Sexp), EvalError> {
    let elements = values::slots(value)?.ok_or_else(|| {
        EvalError::Domain(if level == 0 {
            "attempt to select more than one element in vectorIndex".into()
        } else {
            format!("recursive indexing failed at level {}\n", level + 1)
        })
    })?;
    let index = at(subscript, level)?;
    let position = matching::position(&elements, &index, matching, warn)?;
    let position = position
        .filter(|p| *p < elements.len())
        .ok_or_else(|| EvalError::Domain(format!("no such index at level {}\n", level + 1)))?;
    Ok((position, elements[position].value.clone()))
}

fn exact_position(elements: &[ListElement], index: &Sexp) -> Result<usize, EvalError> {
    if let Some(names) = index.character_values() {
        let name = names[0].as_deref().unwrap_or("NA");
        return Ok(elements
            .iter()
            .position(|slot| !name.is_empty() && slot.name.as_deref() == Some(name))
            .unwrap_or(elements.len()));
    }
    if matches!(index, Sexp::Integer(v) if v[0].is_none()) {
        return super::language::index::read(elements, &Sexp::Integer(vec![Some(i32::MIN)]))?
            .ok_or_else(|| EvalError::Domain("subscript out of bounds".into()));
    }
    if let Sexp::Real(v) = index {
        let value = v[0];
        if value <= -1.0 && !(elements.len() == 2 && value > -3.0) {
            return Err(EvalError::Domain(format!(
                "attempt to select {} than one element in OneIndex <real>",
                if elements.len() < 2 { "less" } else { "more" }
            )));
        }
    }
    match super::language::index::read(elements, index) {
        Ok(position) => Ok(position.unwrap_or(usize::MAX)),
        Err(EvalError::Domain(message)) => Err(EvalError::Domain(
            message.replace("get1index <real>", "OneIndex <real>"),
        )),
        Err(error) => Err(error),
    }
}
