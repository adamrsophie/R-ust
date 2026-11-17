use super::*;

pub(super) fn assign(value: &Sexp, index: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let projected = projection.as_ref();
    if matches!(projected, Sexp::Nil | Sexp::PairList(_))
        || projected.sexptype() == r_ust_sexp::SexpType::Language
    {
        return pair(value, index, replacement);
    }
    let slots = values::slots(value)?;
    let atomic = slots.is_none();
    if atomic {
        match crate::value_len(replacement)? {
            0 => return Err(EvalError::Domain("replacement has length zero".into())),
            1 => {}
            _ => {
                return Err(EvalError::Domain(
                    "more elements supplied than there are to replace".into(),
                ))
            }
        }
    }
    let elements = if let Some(slots) = slots {
        slots
    } else {
        values::scalars(value)?
    };
    let position = exact_position(&elements, index)?;
    if position == usize::MAX {
        return if !atomic && matches!(replacement, Sexp::Nil) {
            Ok(value.clone())
        } else {
            Err(EvalError::Domain("subscript out of bounds".into()))
        };
    }
    if position >= i32::MAX as usize {
        return Err(EvalError::Domain(
            "result would be too long a vector".into(),
        ));
    }
    if matches!(projected, Sexp::Raw(_)) {
        let rhs = crate::coercion::storage::read(replacement)?;
        let Sexp::Raw(rhs) = rhs.as_ref() else {
            return Err(EvalError::Type(format!(
                "incompatible types (from raw to {}) in [[ assignment",
                crate::metadata::typeof_value(rhs.as_ref())
            )));
        };
        let Sexp::Raw(mut raw) = projected.clone() else {
            unreachable!()
        };
        let length = raw.len().max(position + 1);
        raw.try_reserve(length - raw.len())
            .map_err(|_| EvalError::Domain("cannot allocate raw replacement".into()))?;
        raw.resize(length, 0);
        raw[position] = rhs[0];
        return Ok(Sexp::Raw(raw));
    }
    if atomic || projected.ordinary_list().is_some() {
        let normalized = if let Some(names) = index.character_values() {
            Sexp::String(vec![Some(names[0].clone().unwrap_or_else(|| "NA".into()))])
        } else {
            Sexp::Real(vec![position as f64 + 1.0])
        };
        return super::super::extract_assign_value(value, &normalized, replacement);
    }
    let mut slots = elements;
    if matches!(replacement, Sexp::Nil) {
        if position < slots.len() {
            slots.remove(position);
        }
    } else {
        if position >= slots.len() {
            let new_name = index
                .character_values()
                .map(|v| v[0].clone().unwrap_or_else(|| "NA".into()));
            if new_name.is_some() && slots.iter().all(|slot| slot.name.is_none()) {
                for slot in &mut slots {
                    slot.name = Some(String::new());
                }
            }
            slots
                .try_reserve(position + 1 - slots.len())
                .map_err(|_| EvalError::Domain("cannot allocate recursive replacement".into()))?;
            slots.resize_with(position + 1, || ListElement {
                name: None,
                value: Sexp::Nil,
            });
            slots[position].name = new_name;
        }
        slots[position].value = replacement.clone();
    }
    values::restore(value, slots)
}

fn pair(value: &Sexp, index: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    let mut elements = values::slots(value)?.unwrap();
    let deleting = matches!(replacement, Sexp::Nil);
    let unnamed = index
        .character_values()
        .is_some_and(|v| v[0].as_deref().is_none_or(str::is_empty));
    let appended;
    let index = if unnamed && !deleting {
        appended = Sexp::Real(vec![elements.len() as f64 + 1.0]);
        &appended
    } else {
        index
    };
    let selection = super::super::language::selection::select(&elements, index, !deleting)?;
    if deleting {
        let removed_head = selection.positions.contains(&Some(0));
        elements = elements
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !selection.positions.contains(&Some(*i)))
            .map(|(_, slot)| slot)
            .collect();
        return if removed_head && value.sexptype() == r_ust_sexp::SexpType::Language {
            Ok(Sexp::pairlist(elements))
        } else {
            values::restore(value, elements)
        };
    }
    if selection.positions.len() > 1 {
        return Err(EvalError::Domain("invalid subscript in list assign".into()));
    }
    if let Some(Some(position)) = selection.positions.first() {
        let length = position
            .checked_add(1)
            .ok_or_else(|| EvalError::Domain("result would be too long a vector".into()))?;
        if length > elements.len() {
            elements
                .try_reserve(length - elements.len())
                .map_err(|_| EvalError::Domain("cannot allocate language elements".into()))?;
            // Recursive pairlist growth adds unnamed cells, even for character paths.
            elements.resize_with(length, || ListElement {
                name: None,
                value: Sexp::Nil,
            });
        }
        elements[*position].value = replacement.clone();
    }
    values::restore(value, elements)
}
