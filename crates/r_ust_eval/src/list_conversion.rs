use super::*;

pub(super) fn as_list_value(value: &Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::ListObject(_) => Ok(value.clone()),
        Sexp::PairListObject(value) => Ok(crate::list_construction::from_tags(&value.elements()?)),
        Sexp::PairList(value) => Ok(crate::list_construction::from_tags(value.as_slice())),
        Sexp::Nil => Ok(Sexp::List(Vec::new())),
        Sexp::DataFrame { columns, .. } => Ok(Sexp::List(columns.clone())),
        Sexp::ClassedList { .. } | Sexp::NamedList { .. } => Ok(value.clone()),
        Sexp::List(values) | Sexp::Dots(values) => Ok(Sexp::List(values.clone())),
        Sexp::Matrix { data, .. } => as_list_value(data),
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::Factor { .. }
        | Sexp::Vector(_) => Ok(Sexp::List(
            sequence_elements(value)?
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
        )),
        other => Err(EvalError::Type(format!(
            "cannot coerce value of type {:?} to list",
            other.sexptype()
        ))),
    }
}

pub(super) fn flatten_unlist_values(
    value: &Sexp,
    recursive: bool,
    out: &mut Vec<Sexp>,
) -> Result<(), EvalError> {
    if recursive {
        crate::ordinary_cycles::check(value, "cyclic ordinary value coercion")?;
    }
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => {}
        Sexp::DataFrame { columns, .. } if recursive => {
            for element in columns {
                flatten_unlist_values(&element.value, recursive, out)?;
            }
        }
        Sexp::DataFrame { columns, .. } => {
            for element in columns {
                out.push(element.value.clone());
            }
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values)
            if recursive =>
        {
            for element in values {
                flatten_unlist_values(&element.value, recursive, out)?;
            }
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => {
            for element in values {
                out.push(element.value.clone());
            }
        }
        Sexp::Vector(values) if recursive => {
            for value in values {
                flatten_unlist_values(value, recursive, out)?;
            }
        }
        other => out.extend(sequence_elements(other)?),
    }
    Ok(())
}

pub(super) fn call_c(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    if args.iter().all(|arg| matches!(arg.value, Sexp::Nil)) {
        return Ok(Sexp::Nil);
    }
    if args.iter().any(|arg| {
        matches!(
            arg.value,
            Sexp::ClassedList { .. }
                | Sexp::NamedList { .. }
                | Sexp::List(_)
                | Sexp::ListObject(_)
                | Sexp::PairListObject(_)
                | Sexp::PairList(_)
        )
    }) {
        return crate::list_concatenation::concatenate(args);
    }
    if args
        .iter()
        .any(|arg| matches!(&arg.value, Sexp::Factor { .. }))
        && args
            .iter()
            .all(|arg| matches!(&arg.value, Sexp::Nil | Sexp::Factor { .. }))
    {
        return concatenate_factors(args);
    }
    if args.iter().any(|arg| is_character_value(&arg.value)) {
        let mut out = Vec::new();
        for arg in args {
            out.extend(string_values(&arg.value)?);
        }
        return Ok(Sexp::String(out));
    }
    if args.iter().any(|arg| is_double_value(&arg.value)) {
        return Ok(Sexp::Real(flatten_numeric_args(args)?));
    }
    if args.iter().any(|arg| typeof_value(&arg.value) == "integer") {
        return Ok(Sexp::Integer(
            flatten_numeric_args(args)?
                .into_iter()
                .map(real_to_integer)
                .collect(),
        ));
    }
    let mut out = Vec::new();
    for arg in args {
        out.extend(logical_values(&arg.value)?);
    }
    Ok(Sexp::Logical(out))
}

pub(super) fn call_unlist(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    let recursive = named_bool_arg(args, &["recursive"], true)?;
    if value.character_values().is_some() {
        return Ok(value.clone());
    }
    let mut values = Vec::new();
    flatten_unlist_values(value, recursive, &mut values)?;
    unlisted_values(values)
}

pub(super) fn unlisted_values(values: Vec<Sexp>) -> Result<Sexp, EvalError> {
    scalar_elements_to_vector(&values)
}
