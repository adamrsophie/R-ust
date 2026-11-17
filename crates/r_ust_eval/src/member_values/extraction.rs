use super::NameMatching;
use crate::{extraction_position, EvalError, Sexp};

pub(crate) fn extract_value(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    extract_matching(value, index, NameMatching::Exact, &mut |_, _| Ok(()))
}

pub(crate) fn extract_matching(
    value: &Sexp,
    index: &Sexp,
    matching: NameMatching,
    warn: &mut impl FnMut(&str, bool) -> Result<(), EvalError>,
) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    let subscript = crate::coercion::storage::read(index)?;
    let index = subscript.as_ref();
    if matches!(value, Sexp::Nil) && !matches!(index, Sexp::MissingValue | Sexp::MissingArg(_)) {
        return Ok(Sexp::Nil);
    }
    if crate::value_len(index)? > 1 {
        return super::recursive::extract(value, index, matching, warn);
    }
    if index.character_values().is_some() || matches!(index, Sexp::Symbol(_)) {
        return super::recursive::extract(value, index, matching, warn);
    }
    if matches!(value, Sexp::AttributedAggregate { .. }) {
        return super::typed::extract(value, index);
    }
    if value.sexptype() == r_ust_sexp::SexpType::Language {
        return super::language::extract(value, index);
    }
    if let Sexp::Expression {
        elements,
        names_present,
    } = value
    {
        return crate::expression_values::extract((elements, *names_present), index);
    }
    if let Sexp::PairList(values) = value {
        return extract_value(&Sexp::List(values.as_slice().to_vec()), index);
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
        | Sexp::NamedList {
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
