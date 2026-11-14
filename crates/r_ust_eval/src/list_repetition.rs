use super::*;

pub(super) fn call_rep(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    let times = numeric_arg_scalar(args, 1, &["times"], Some(1.0))?
        .trunc()
        .max(0.0) as usize;

    match value {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Logical(values) => Ok(Sexp::Logical(repeat_slice(values, times))),
        Sexp::Integer(values) => Ok(Sexp::Integer(repeat_slice(values, times))),
        Sexp::Real(values) => Ok(Sexp::Real(repeat_slice(values, times))),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            Ok(Sexp::String(repeat_slice(values, times)))
        }
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(Sexp::Factor {
            codes: repeat_slice(codes, times),
            levels: levels.clone(),
            ordered: *ordered,
        }),
        Sexp::Vector(values) => Ok(Sexp::Vector(repeat_slice(values, times))),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => Ok(Sexp::List(repeat_slice(values, times))),
        other => Err(EvalError::Type(format!(
            "cannot rep value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn call_rep_len(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    let length_out = numeric_arg_scalar(args, 1, &["length.out"], None)?
        .trunc()
        .max(0.0) as usize;
    if length_out == 0 {
        return Ok(match value {
            Sexp::Nil => Sexp::Nil,
            Sexp::Logical(_) => Sexp::Logical(Vec::new()),
            Sexp::Integer(_) => Sexp::Integer(Vec::new()),
            Sexp::Real(_) => Sexp::Real(Vec::new()),
            Sexp::String(_) | Sexp::AttributedCharacter { .. } => Sexp::String(Vec::new()),
            Sexp::Factor {
                levels, ordered, ..
            } => Sexp::Factor {
                codes: Vec::new(),
                levels: levels.clone(),
                ordered: *ordered,
            },
            Sexp::ClassedList { .. } | Sexp::List(_) => Sexp::List(Vec::new()),
            Sexp::Vector(_) => Sexp::Vector(Vec::new()),
            _ => Sexp::Nil,
        });
    }
    match value {
        Sexp::Nil => Err(EvalError::Arity(
            "cannot replicate NULL to a non-zero length".to_string(),
        )),
        Sexp::Logical(values) => Ok(Sexp::Logical(repeat_to_length(values, length_out)?)),
        Sexp::Integer(values) => Ok(Sexp::Integer(repeat_to_length(values, length_out)?)),
        Sexp::Real(values) => Ok(Sexp::Real(repeat_to_length(values, length_out)?)),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            Ok(Sexp::String(repeat_to_length(values, length_out)?))
        }
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(Sexp::Factor {
            codes: repeat_to_length(codes, length_out)?,
            levels: levels.clone(),
            ordered: *ordered,
        }),
        Sexp::Vector(values) => Ok(Sexp::Vector(repeat_to_length(values, length_out)?)),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => Ok(Sexp::List(repeat_to_length(values, length_out)?)),
        other => Err(EvalError::Type(format!(
            "cannot rep_len value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn list_like_elements(value: &Sexp) -> Result<Option<Vec<ListElement>>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    Ok(match value {
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Some(values.clone()),
        Sexp::DataFrame { columns, .. } => Some(columns.clone()),
        Sexp::PairList(values) => Some(values.as_slice().to_vec()),
        _ => None,
    })
}

pub(super) fn lengths_values(value: &Sexp) -> Result<Vec<usize>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Some(elements) = list_like_elements(value)? {
        return elements
            .iter()
            .map(|element| value_len(&element.value))
            .collect();
    }
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Vector(values) => values.iter().map(value_len).collect(),
        Sexp::Matrix { data, .. } => Ok(vec![1; value_len(data)?]),
        _ => Ok(vec![1; value_len(value)?]),
    }
}
