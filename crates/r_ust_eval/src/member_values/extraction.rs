use crate::{extraction_position, string_scalar, EvalError, Sexp};

pub(crate) fn extract_value(value: &Sexp, index: &Sexp) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
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
    if let Ok(name) = string_scalar(index, "extraction index") {
        if name.is_empty() && value.ordinary_list().is_some() {
            return Ok(Sexp::Nil);
        }
        return match value {
            Sexp::DataFrame { columns, .. } => Ok(columns
                .iter()
                .find(|element| element.name.as_deref() == Some(name.as_str()))
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
