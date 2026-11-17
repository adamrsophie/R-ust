use super::*;

pub(super) fn one_dimensional(attrs: &[ListElement]) -> Result<bool, EvalError> {
    Ok(
        matches!(find(attrs,"dim").map(payload).transpose()?,Some(Sexp::Integer(values)) if values.len()==1),
    )
}

pub(super) fn dims(object: &Sexp, value: &Sexp) -> Result<Sexp, EvalError> {
    let size = length(object)?.ok_or_else(|| {
        EvalError::Type("invalid first argument, must be vector (list or atomic)".into())
    })?;
    let plain = payload(value)?;
    if !matches!(
        plain,
        Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_)
    ) {
        return Err(EvalError::Type(
            "invalid second argument, must be vector or NULL".into(),
        ));
    }
    let dimensions = crate::coerce_integer_values(&plain)?;
    if dimensions.is_empty() {
        return Err(EvalError::Domain(
            "length-0 dimension vector is invalid".into(),
        ));
    }
    let mut total = 1usize;
    for dim in &dimensions {
        let dim = dim.ok_or_else(|| EvalError::Domain("the dims contain missing values".into()))?;
        let dim = usize::try_from(dim)
            .map_err(|_| EvalError::Domain("the dims contain negative values".into()))?;
        total = total
            .checked_mul(dim)
            .ok_or_else(|| EvalError::Domain("dimension product is too large".into()))?;
    }
    if total != size {
        return Err(EvalError::Domain(format!(
            "dims [product {total}] do not match the length of object [{size}]"
        )));
    }
    Ok(Sexp::Integer(dimensions))
}

pub(super) fn names(object: &Sexp, value: &Sexp) -> Result<Sexp, EvalError> {
    let attrs = stored(object)?;
    let dim = find(&attrs, "dim")
        .ok_or_else(|| EvalError::Domain("'dimnames' applied to non-array".into()))?;
    let dimensions = crate::coerce_integer_values(&payload(dim)?)?;
    let plain = if let Sexp::Managed(object) = value {
        object.borrow()?.value().clone()
    } else {
        value.clone()
    };
    let mut entries = if let Sexp::PairList(entries) = &plain {
        entries.as_slice().to_vec()
    } else {
        list(&plain).map_err(|_| EvalError::Type("'dimnames' must be a list".into()))?
    };
    if entries.len() > dimensions.len() {
        return Err(EvalError::Domain(format!(
            "length of 'dimnames' [{}] must match that of 'dims' [{}]",
            entries.len(),
            dimensions.len()
        )));
    }
    if entries.is_empty() {
        return Ok(Sexp::Nil);
    }
    let attributes = replacements::resized_attributes(&plain, (entries.len(), dimensions.len()))?;
    entries.resize_with(dimensions.len(), || ListElement {
        name: None,
        value: Sexp::Nil,
    });
    for (i, entry) in entries.iter_mut().enumerate() {
        if matches!(payload(&entry.value)?, Sexp::Nil) {
            continue;
        }
        let size = length(&entry.value)?.ok_or_else(|| {
            EvalError::Type(format!(
                "invalid type ({}) for 'dimnames' (must be a vector)",
                crate::metadata::typeof_value(&entry.value)
            ))
        })?;
        let expected = dimensions[i]
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0);
        if size != 0 && size != expected {
            return Err(EvalError::Domain(format!(
                "length of 'dimnames' [{}] not equal to array extent",
                i + 1
            )));
        }
        entry.value = if size == 0 {
            Sexp::Nil
        } else {
            simple_character(&entry.value)?
        };
    }
    let result = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(entries.into_iter().map(|entry| entry.value).collect()),
        attributes,
    };
    restore::value(&plain, result)
}

pub(super) fn simple_character(value: &Sexp) -> Result<Sexp, EvalError> {
    let attrs = stored(value)?;
    let plain = payload(value)?;
    let class = find(&attrs, "class").map(payload).transpose()?;
    if class
        .as_ref()
        .and_then(Sexp::character_values)
        .is_some_and(|classes| classes.iter().any(|c| c.as_deref() == Some("factor")))
    {
        let Sexp::Integer(codes) = plain else {
            return Err(EvalError::Type("malformed factor".into()));
        };
        let levels =
            find(&attrs, "levels").ok_or_else(|| EvalError::Type("malformed factor".into()))?;
        let levels = payload(levels)?;
        let levels = levels
            .character_values()
            .ok_or_else(|| EvalError::Type("malformed factor".into()))?;
        let values = codes
            .into_iter()
            .map(|code| match code {
                None => Ok(None),
                Some(code) => code
                    .checked_sub(1)
                    .and_then(|code| usize::try_from(code).ok())
                    .and_then(|i| levels.get(i))
                    .cloned()
                    .ok_or_else(|| EvalError::Type("malformed factor".into())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Sexp::String(values));
    }
    if plain.character_values().is_some() {
        return Ok(value.clone());
    }
    Ok(Sexp::String(crate::string_values(&plain)?))
}
