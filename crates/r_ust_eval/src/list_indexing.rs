use super::*;

pub(super) fn subset_value(value: &Sexp, index: Option<&Sexp>) -> Result<Sexp, EvalError> {
    character_attributes::validate(value)?;
    match value {
        Sexp::PairList(values) => {
            return subset_value(&Sexp::List(values.as_slice().to_vec()), index)
        }
        Sexp::PairListObject(values) => {
            return subset_value(&Sexp::List(values.elements()?), index)
        }
        _ => {}
    }
    let Some(index) = index else {
        return Ok(value.clone());
    };
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let (Sexp::DataFrame { columns, nrow }, Sexp::String(_) | Sexp::AttributedCharacter { .. }) =
        (value, index)
    {
        return Ok(Sexp::DataFrame {
            columns: subset_list_by_names(columns, &string_values(index)?),
            nrow: *nrow,
        });
    }
    if let (
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values),
        Sexp::String(_) | Sexp::AttributedCharacter { .. },
    ) = (value, index)
    {
        return Ok(Sexp::List(subset_list_by_names(
            values,
            &string_values(index)?,
        )));
    }
    let positions = subset_positions(value_len(value)?, index)?;

    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::Factor { .. }
        | Sexp::Vector(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_) => subset_linear_positions(value, &positions),
        Sexp::DataFrame { columns, nrow } => Ok(Sexp::DataFrame {
            columns: subset_list(columns, &positions),
            nrow: *nrow,
        }),
        Sexp::Matrix { data, .. } => subset_linear_positions(data, &positions),
        other => Err(EvalError::Type(format!(
            "cannot subset value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn subset_linear_positions(
    value: &Sexp,
    positions: &[Option<usize>],
) -> Result<Sexp, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(values) => Ok(Sexp::Logical(subset_optional(values, positions))),
        Sexp::Integer(values) => Ok(Sexp::Integer(subset_optional(values, positions))),
        Sexp::Real(values) => Ok(Sexp::Real(subset_real(values, positions))),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            Ok(Sexp::String(subset_optional(values, positions)))
        }
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(subset_factor_positions(codes, levels, *ordered, positions)),
        Sexp::Vector(values) => Ok(Sexp::Vector(subset_vector(values, positions))),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => Ok(Sexp::List(subset_list(values, positions))),
        Sexp::Matrix { data, .. } => subset_linear_positions(data, positions),
        other => Err(EvalError::Type(format!(
            "cannot subset value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn subassign_value(
    value: &Sexp,
    index: Option<&Sexp>,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    character_attributes::validate(value)?;
    if let Sexp::ListObject(object) = value {
        let replacement = subassign_value(&object.value()?, index, replacement)?;
        return ordinary_values::replaced(value, replacement);
    }
    match value {
        Sexp::PairList(values) => {
            return subassign_value(&Sexp::List(values.as_slice().to_vec()), index, replacement)
        }
        Sexp::PairListObject(values) => {
            return subassign_value(&Sexp::List(values.elements()?), index, replacement)
        }
        _ => {}
    }
    let positions = assignment_positions(index, value_len(value)?)?;
    if positions.is_empty() {
        return Ok(value.clone());
    }

    if matches!(value, Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_))
        && is_character_value(replacement)
    {
        let mut out = string_values(value)?;
        assign_optional_values(&mut out, &positions, &string_values(replacement)?)?;
        return Ok(Sexp::String(out));
    }
    if matches!(value, Sexp::Logical(_) | Sexp::Integer(_)) && is_double_value(replacement) {
        let mut out = numeric_values(value)?;
        assign_real_values(&mut out, &positions, &numeric_values(replacement)?)?;
        return Ok(Sexp::Real(out));
    }
    if matches!(value, Sexp::Logical(_)) && typeof_value(replacement) == "integer" {
        let mut out = coerce_integer_values(value)?;
        assign_optional_values(&mut out, &positions, &coerce_integer_values(replacement)?)?;
        return Ok(Sexp::Integer(out));
    }

    match value {
        Sexp::Nil => subassign_nil(&positions, replacement),
        Sexp::Logical(values) => {
            let mut out = values.clone();
            let replacement = logical_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(Sexp::Logical(out))
        }
        Sexp::Integer(values) => {
            let mut out = values.clone();
            let replacement = coerce_integer_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(Sexp::Integer(out))
        }
        Sexp::Real(values) => {
            let mut out = values.clone();
            assign_real_values(&mut out, &positions, &numeric_values(replacement)?)?;
            Ok(Sexp::Real(out))
        }
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            let mut out = values.clone();
            let replacement = string_values(replacement)?;
            assign_optional_values(&mut out, &positions, &replacement)?;
            Ok(character_attributes::with_values(value, out))
        }
        Sexp::Vector(values) => {
            let mut out = values.clone();
            assign_vector_values(&mut out, &positions, &sequence_elements(replacement)?)?;
            Ok(Sexp::Vector(out))
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let mut out = values.clone();
            assign_list_values(&mut out, &positions, &list_replacement_values(replacement)?)?;
            Ok(list_attributes::preserve_class(value, out))
        }
        Sexp::Matrix { data, nrow, ncol } => {
            let data = subassign_value(data, index, replacement)?;
            Ok(Sexp::Matrix {
                data: Box::new(data),
                nrow: *nrow,
                ncol: *ncol,
            })
        }
        other => Err(EvalError::Type(format!(
            "cannot assign through subset of type {:?}",
            other.sexptype()
        ))),
    }
}
