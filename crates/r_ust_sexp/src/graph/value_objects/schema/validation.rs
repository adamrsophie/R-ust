use super::*;

fn text(value: &str) -> Result<(), GraphError> {
    if value.contains('\0') {
        Err(GraphError::InvalidCharacter)
    } else {
        Ok(())
    }
}

fn names(values: &[ListElement]) -> Result<(), GraphError> {
    for value in values {
        if let Some(name) = &value.name {
            text(name)?;
        }
    }
    Ok(())
}

fn length(value: &Sexp) -> Result<usize, GraphError> {
    match value {
        Sexp::Nil => Ok(0),
        Sexp::Logical(v) => Ok(v.len()),
        Sexp::Integer(v) => Ok(v.len()),
        Sexp::Real(v) => Ok(v.len()),
        Sexp::String(v) | Sexp::AttributedCharacter { values: v, .. } => Ok(v.len()),
        Sexp::Factor { codes, .. } => Ok(codes.len()),
        Sexp::Vector(v) => Ok(v.len()),
        Sexp::List(v)
        | Sexp::NamedList { elements: v, .. }
        | Sexp::ClassedList { elements: v, .. }
        | Sexp::Expression { elements: v, .. } => Ok(v.len()),
        Sexp::Matrix { data, .. } => length(data),
        _ => Err(GraphError::WrongKind),
    }
}

pub(in crate::graph::value_objects) fn validate_shape(value: &Sexp) -> Result<(), GraphError> {
    match value {
        Sexp::String(v) | Sexp::AttributedCharacter { values: v, .. } => {
            for s in v.iter().flatten() {
                text(s)?;
            }
        }
        Sexp::Char(Some(s)) | Sexp::Symbol(s) | Sexp::MissingArg(s) => text(s)?,
        Sexp::Factor { codes, levels, .. } => {
            for s in levels {
                text(s)?;
            }
            if codes
                .iter()
                .flatten()
                .any(|code| *code == 0 || *code > levels.len())
            {
                return Err(GraphError::WrongKind);
            }
        }
        Sexp::Closure { parameters, .. } | Sexp::Function { parameters, .. } => {
            for p in parameters {
                text(&p.name)?;
            }
        }
        _ => {}
    }
    match value {
        Sexp::Closure { body, .. }
            if matches!(
                body.sexptype(),
                SexpType::Closure
                    | SexpType::Builtin
                    | SexpType::Special
                    | SexpType::Dots
                    | SexpType::Any
            ) =>
        {
            return Err(GraphError::WrongKind)
        }
        Sexp::For { variable, .. } => text(variable)?,
        Sexp::Assignment { target, .. }
        | Sexp::SuperAssignment { target, .. }
        | Sexp::Subassignment { target, .. }
        | Sexp::MatrixSubassignment { target, .. }
        | Sexp::ExtractAssignment { target, .. }
        | Sexp::NamesAssignment { target, .. } => text(target)?,
        Sexp::Dollar { name, .. } => text(name.as_str())?,
        Sexp::DollarAssignment { target, name, .. } => {
            text(target)?;
            text(name.as_str())?;
        }
        _ => {}
    }
    match value {
        Sexp::List(v)
        | Sexp::Dots(v)
        | Sexp::NamedList { elements: v, .. }
        | Sexp::ClassedList { elements: v, .. }
        | Sexp::Expression { elements: v, .. }
        | Sexp::AttributedCharacter { attributes: v, .. }
        | Sexp::Closure { attributes: v, .. } => names(v)?,
        Sexp::PairList(v) => names(v.as_slice())?,
        Sexp::DataFrame { columns, nrow } => {
            names(columns)?;
            for column in columns {
                if length(&column.value)? != *nrow {
                    return Err(GraphError::WrongKind);
                }
            }
        }
        Sexp::Matrix { data, nrow, ncol } => {
            if nrow.checked_mul(*ncol) != Some(length(data)?) {
                return Err(GraphError::WrongKind);
            }
        }
        Sexp::Call { arguments, .. }
        | Sexp::InternalCall { arguments, .. }
        | Sexp::ExtractArguments { arguments, .. } => {
            for arg in arguments {
                if let Some(name) = &arg.name {
                    text(name)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
