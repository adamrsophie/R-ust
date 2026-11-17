use crate::{EvalError, ListElement, Sexp};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameMatching {
    Exact,
    Partial,
    Warn,
}

pub(super) fn position(
    elements: &[ListElement],
    index: &Sexp,
    matching: NameMatching,
    warn: &mut impl FnMut(&str, bool) -> Result<(), EvalError>,
) -> Result<Option<usize>, EvalError> {
    let exact = super::language::index::read(elements, index)?;
    if exact.is_some() || matching == NameMatching::Exact {
        return Ok(exact);
    }
    let Some(Some(name)) = index.character_values().and_then(|v| v.first()) else {
        return Ok(None);
    };
    if name.is_empty() {
        return Ok(None);
    }
    let mut selected = None;
    for (i, slot) in elements.iter().enumerate() {
        let Some(full) = slot.name.as_deref().filter(|full| full.starts_with(name)) else {
            continue;
        };
        let first = selected.is_none();
        if matching == NameMatching::Warn {
            let prefix = if first { "partial" } else { "further partial" };
            warn(&format!("{prefix} match of '{name}' to '{full}'"), first)?;
        }
        if !first {
            return Ok(None);
        }
        selected = Some(i);
    }
    Ok(selected)
}
