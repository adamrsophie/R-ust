use crate::{EvalError, ListElement, Sexp};

pub(super) fn render(
    value: &Sexp,
    attributes: &[ListElement],
    quote: bool,
) -> Result<String, EvalError> {
    let Some(dimensions) = super::dimensions::read(attributes)? else {
        let names = super::metadata::get(attributes, "names")?
            .map(|v| super::metadata::strings(&v))
            .transpose()?;
        return super::vector::render(value, names.as_deref(), quote);
    };
    if dimensions.sizes.len() == 1 {
        let mut result = super::vector::render(value, dimensions.labels[0].as_deref(), quote)?;
        if dimensions.labels[0].is_some() {
            if let Some(Some(title)) = dimensions.titles.first() {
                result = format!("{title}\n{result}");
            }
        }
        return Ok(result);
    }
    let length = dimensions
        .sizes
        .iter()
        .try_fold(1usize, |n, d| n.checked_mul(*d))
        .ok_or_else(|| EvalError::Type("print dimensions overflow".into()))?;
    let rows = dimensions.sizes[0];
    let columns = dimensions.sizes[1];
    let cells = rows
        .checked_mul(columns)
        .ok_or_else(|| EvalError::Type("print dimensions overflow".into()))?;
    let (values, _, _) = super::numeric::tokens(value, quote)?;
    if values.len() != length {
        return Err(EvalError::Type("print dimensions mismatch".into()));
    }
    if dimensions.sizes.len() == 2 {
        if rows == 0 && columns == 0 {
            return Ok("<0 x 0 matrix>".into());
        }
        return super::matrix::render(value, &dimensions, (Some(0), quote));
    }
    let slices = dimensions.sizes[2..].iter().product::<usize>();
    if slices == 0 {
        let kind = match value {
            Sexp::Raw(_) | Sexp::Logical(_) => "logical",
            Sexp::Integer(_) => "integer",
            Sexp::Real(_) => "double",
            Sexp::String(_) => "character",
            _ => return Err(EvalError::Type("array print type".into())),
        };
        return Ok(format!(
            "<{} array of {kind}>\n{}\n",
            dimensions
                .sizes
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" x "),
            super::matrix::render(value, &dimensions, (None, quote))?
        ));
    }
    let mut result = String::new();
    for slice in 0..slices {
        let mut rest = slice;
        let mut label = ", ".to_string();
        for axis in 2..dimensions.sizes.len() {
            let index = rest % dimensions.sizes[axis];
            rest /= dimensions.sizes[axis];
            let value = dimensions.labels[axis]
                .as_ref()
                .map(|v| v[index].as_deref().unwrap_or("<NA>").to_string())
                .unwrap_or_else(|| (index + 1).to_string());
            label.push_str(", ");
            if dimensions.labels[axis].is_some() {
                if let Some(Some(title)) = dimensions.titles.get(axis) {
                    label.push_str(title);
                    label.push_str(" = ");
                }
            }
            label.push_str(&value);
        }
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&label);
        result.push_str("\n\n");
        result.push_str(&super::matrix::render(
            value,
            &dimensions,
            (Some(slice * cells), quote),
        )?);
        result.push('\n');
    }
    Ok(result)
}
