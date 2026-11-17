use super::*;
use std::collections::HashSet;

pub(super) fn validate(
    kind: SexpType,
    length: usize,
    attributes: &[ListElement],
) -> Result<bool, GraphError> {
    let mut names = HashSet::new();
    let mut object = false;
    let mut dimensions = None;
    for attribute in attributes {
        let name = attribute.name.as_deref().ok_or(GraphError::WrongKind)?;
        if name.contains('\0') {
            return Err(GraphError::InvalidCharacter);
        }
        if name.is_empty() || !names.insert(name) {
            return Err(GraphError::WrongKind);
        }
        match name {
            "names" | "class" => {
                let value = attribute.value.snapshot()?;
                let text = value.character_values().ok_or(GraphError::WrongKind)?;
                if name == "names" {
                    if text.len() != length {
                        return Err(GraphError::WrongKind);
                    }
                } else {
                    if text.is_empty()
                        || (text.iter().flatten().any(|class| class == "factor")
                            && kind != SexpType::Integer)
                    {
                        return Err(GraphError::WrongKind);
                    }
                    object = true;
                }
            }
            "dim" => {
                let dims = integer(&attribute.value)?;
                if dims.is_empty() {
                    return Err(GraphError::WrongKind);
                }
                let size = dims.iter().try_fold(1usize, |size, dimension| {
                    let dimension = usize::try_from(dimension.ok_or(GraphError::WrongKind)?)
                        .map_err(|_| GraphError::WrongKind)?;
                    size.checked_mul(dimension).ok_or(GraphError::WrongKind)
                })?;
                if size != length {
                    return Err(GraphError::WrongKind);
                }
                dimensions = Some(dims);
            }
            "row.names" => rows(&attribute.value)?,
            _ => {}
        }
    }
    if let Some(names) = attributes
        .iter()
        .find(|a| a.name.as_deref() == Some("dimnames"))
    {
        dimnames(
            &names.value,
            dimensions.as_deref().ok_or(GraphError::WrongKind)?,
        )?;
    }
    Ok(object)
}

fn integer(value: &Sexp) -> Result<Vec<Option<i32>>, GraphError> {
    match value.snapshot()? {
        Sexp::Integer(values)
        | Sexp::AttributedAtomic {
            values: AtomicVector::Integer(values),
            ..
        } => Ok(values),
        _ => Err(GraphError::WrongKind),
    }
}

fn rows(value: &Sexp) -> Result<(), GraphError> {
    match value.snapshot()? {
        Sexp::String(_)
        | Sexp::AttributedCharacter { .. }
        | Sexp::AttributedAtomic {
            values: AtomicVector::String(_),
            ..
        } => Ok(()),
        Sexp::Integer(values)
        | Sexp::AttributedAtomic {
            values: AtomicVector::Integer(values),
            ..
        } => {
            if let [None, count] = values.as_slice() {
                count
                    .ok_or(GraphError::WrongKind)?
                    .checked_abs()
                    .ok_or(GraphError::WrongKind)?;
            }
            Ok(())
        }
        _ => Err(GraphError::WrongKind),
    }
}

fn dimnames(value: &Sexp, dimensions: &[Option<i32>]) -> Result<(), GraphError> {
    let value = value.snapshot()?;
    let values = match &value {
        Sexp::ListObject(value) => return dimnames(&value.value()?, dimensions),
        Sexp::Vector(values)
        | Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            ..
        } => values.clone(),
        value => value
            .ordinary_list()
            .ok_or(GraphError::WrongKind)?
            .elements
            .iter()
            .map(|e| e.value.clone())
            .collect(),
    };
    if values.len() != dimensions.len() {
        return Err(GraphError::WrongKind);
    }
    for (value, dimension) in values.iter().zip(dimensions) {
        if *value == Sexp::Nil {
            continue;
        }
        let text = value.character_values().ok_or(GraphError::WrongKind)?;
        let length = usize::try_from(dimension.ok_or(GraphError::WrongKind)?)
            .map_err(|_| GraphError::WrongKind)?;
        if text.len() != length {
            return Err(GraphError::WrongKind);
        }
    }
    Ok(())
}
