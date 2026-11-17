use crate::{EvalError, ListElement};

pub(super) fn read(attributes: &[ListElement]) -> Result<Option<Vec<Option<String>>>, EvalError> {
    if let Some(value) = super::metadata::get(attributes, "names")? {
        return super::metadata::strings(&value).map(Some);
    }
    if let Some(dim) = super::dimensions::read(attributes)? {
        if dim.sizes.len() == 1 {
            return Ok(dim.labels.into_iter().next().flatten());
        }
    }
    Ok(None)
}
