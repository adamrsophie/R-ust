use super::*;

pub(in crate::member_values) fn assign(
    value: &Sexp,
    subscript: &Sexp,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    let length = crate::value_len(subscript)?;
    if matches!(value, Sexp::Nil) && matches!(replacement, Sexp::Nil) {
        return Ok(Sexp::Nil);
    }
    let mut current = value.clone();
    let mut frames = Vec::new();
    for level in 0..length - 1 {
        if level == length - 2 {
            if let Some(slots) = values::slots(&current)? {
                exact_position(&slots, &at(subscript, level)?)?;
            }
        }
        let (position, child) = descend(&current, subscript, (level, true))?;
        frames.push((current, position));
        current = child;
    }
    let leaf_index = at(subscript, length - 1)?;
    let original_leaf = current.clone();
    let projection = crate::ordinary_values::read(&current)?;
    let projected = projection.as_ref();
    let mut updated = if values::slots(projected)?.is_none()
        && !matches!(
            projected,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::Raw(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::AttributedAtomic { .. }
                | Sexp::Factor { .. }
                | Sexp::Matrix { .. }
        ) {
        return Err(EvalError::Type(format!(
            "object of type '{}' is not subsettable",
            crate::metadata::typeof_value(projected)
        )));
    } else {
        leaf::assign(&current, &leaf_index, replacement)?
    };
    for (depth, (parent, selected)) in frames.into_iter().enumerate().rev() {
        let mut slots = values::slots(&parent)?.unwrap();
        let target = if depth == length - 2 {
            exact_position(&slots, &at(subscript, depth)?)?
        } else {
            selected
        };
        if target >= slots.len() {
            let projection = crate::ordinary_values::read(&parent)?;
            if !matches!(projection.as_ref(), Sexp::Nil | Sexp::PairList(_))
                && parent.sexptype() != r_ust_sexp::SexpType::Language
            {
                return Err(EvalError::Domain(format!(
                    "attempt to set index {}/{} in SET_VECTOR_ELT",
                    target,
                    slots.len()
                )));
            }
            let same_type = original_leaf.sexptype() == updated.sexptype();
            let same_size =
                same_type && crate::value_len(&original_leaf)? == crate::value_len(&updated)?;
            let same_head =
                if same_type && original_leaf.sexptype() == r_ust_sexp::SexpType::Language {
                    true
                } else if same_type && original_leaf.sexptype() == r_ust_sexp::SexpType::List {
                    let removed_head = matches!(replacement, Sexp::Nil)
                        && super::super::language::selection::select(
                            &values::slots(&original_leaf)?.unwrap(),
                            &leaf_index,
                            false,
                        )?
                        .positions
                        .contains(&Some(0));
                    !removed_head
                } else {
                    false
                };
            // Retain leaf mutations before R's separate exact parent writeback.
            if same_size || same_head {
                slots[selected].value = updated.clone();
            }
            slots.resize_with(target + 1, || ListElement {
                name: None,
                value: Sexp::Nil,
            });
        }
        slots[target].value = updated;
        updated = values::restore(&parent, slots)?;
    }
    Ok(updated)
}
