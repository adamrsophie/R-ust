use crate::{EvalError, ListElement, Sexp};

pub(super) struct Selection {
    pub(super) positions: Vec<Option<usize>>,
    pub(super) new_tag: Option<String>,
}

pub(super) fn select(
    elements: &[ListElement],
    index: &Sexp,
    extend: bool,
) -> Result<Selection, EvalError> {
    if let Sexp::Matrix { data, .. } = index {
        return select(elements, data, extend);
    }
    match crate::value_len(index)? {
        0 => {
            return Err(EvalError::Domain(
                "internal error: index -1 from length 0".into(),
            ))
        }
        1 => {}
        _ => return Err(EvalError::NotImplemented("recursive language assignment")),
    }
    if let Some(names) = index.character_values() {
        return named(elements, names[0].as_deref(), extend);
    }
    let positions = match index {
        Sexp::Logical(_) => crate::subset_positions(elements.len(), index)?,
        Sexp::Integer(_) | Sexp::Real(_) | Sexp::Factor { .. } => {
            numeric(elements.len(), index, extend)?
        }
        Sexp::MissingValue | Sexp::MissingArg(_) => (0..elements.len()).map(Some).collect(),
        _ => return Err(super::index::invalid_type(index)),
    };
    Ok(Selection {
        positions,
        new_tag: None,
    })
}

fn named(
    elements: &[ListElement],
    name: Option<&str>,
    extend: bool,
) -> Result<Selection, EvalError> {
    let existing = name.filter(|name| !name.is_empty()).and_then(|name| {
        elements
            .iter()
            .position(|slot| slot.name.as_deref() == Some(name))
    });
    if let Some(index) = existing {
        return Ok(Selection {
            positions: vec![Some(index)],
            new_tag: None,
        });
    }
    if !extend {
        return Err(EvalError::Domain("subscript out of bounds".into()));
    }
    let name = name.unwrap_or("NA");
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    Ok(Selection {
        positions: vec![Some(elements.len())],
        new_tag: Some(name.into()),
    })
}

fn numeric(length: usize, index: &Sexp, extend: bool) -> Result<Vec<Option<usize>>, EvalError> {
    let indices = crate::numeric_index_values(index)?;
    let Some(value) = indices[0] else {
        return Ok(vec![None]);
    };
    if value < 0 {
        let excluded = value.unsigned_abs();
        return Ok((0..length)
            .filter(|position| position + 1 != excluded)
            .map(Some)
            .collect());
    }
    if value == 0 {
        return Ok(Vec::new());
    }
    let position = usize::try_from(value - 1)
        .map_err(|_| EvalError::Domain("subscript out of bounds".into()))?;
    if !extend && position >= length {
        return Err(EvalError::Domain("subscript out of bounds".into()));
    }
    if position
        >= usize::try_from(i32::MAX)
            .map_err(|_| EvalError::Domain("invalid pairlist limit".into()))?
    {
        return Err(EvalError::Domain(
            "result would be too long a vector".into(),
        ));
    }
    Ok(vec![Some(position)])
}
