use super::*;

pub(super) fn sequence_elements(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::PairList(values) => Ok(values
            .as_slice()
            .iter()
            .map(|slot| slot.value.clone())
            .collect()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Logical(vec![value]))
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Integer(vec![value]))
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .copied()
            .map(|value| Sexp::Real(vec![value]))
            .collect()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => Ok(values
            .iter()
            .cloned()
            .map(|value| Sexp::String(vec![value]))
            .collect()),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => Ok(factor_elements(codes, levels, *ordered)),
        Sexp::DataFrame { columns, .. } => Ok(columns
            .iter()
            .map(|element| element.value.clone())
            .collect()),
        Sexp::Matrix { data, .. } => sequence_elements(data),
        Sexp::Vector(values) => Ok(values.clone()),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(values.iter().map(|element| element.value.clone()).collect()),
        other => Err(EvalError::Type(format!(
            "cannot iterate over value of type {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn apply_elements(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Nil => Ok(Vec::new()),
        Sexp::PairList(values) => Ok(values.as_slice().to_vec()),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(values.clone()),
        Sexp::DataFrame { columns, .. } => Ok(columns.clone()),
        other => Ok(sequence_elements(other)?
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect()),
    }
}

pub(super) fn list_replacement_values(value: &Sexp) -> Result<Vec<Sexp>, EvalError> {
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values) => Ok(values.iter().map(|element| element.value.clone()).collect()),
        _ => sequence_elements(value),
    }
}

pub(super) fn data_mask_scope(data: &Sexp) -> Result<HashMap<String, Sexp>, EvalError> {
    let projection = ordinary_values::read(data)?;
    let data = projection.as_ref();
    let mut scope = HashMap::new();
    match data {
        Sexp::DataFrame { columns, .. }
        | Sexp::ClassedList {
            elements: columns, ..
        }
        | Sexp::List(columns)
        | Sexp::Dots(columns) => {
            for element in columns {
                if let Some(name) = &element.name {
                    scope.insert(name.clone(), element.value.clone());
                }
            }
            Ok(scope)
        }
        other => Err(EvalError::Type(format!(
            "data mask expected a data frame or named list, found {:?}",
            other.sexptype()
        ))),
    }
}
