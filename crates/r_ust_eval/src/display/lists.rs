use super::format_sexp;
use crate::{EvalError, ListElement, Sexp};

pub(super) fn format_list(values: &[ListElement]) -> Result<String, EvalError> {
    format_nested(values, "")
}

fn format_nested(values: &[ListElement], prefix: &str) -> Result<String, EvalError> {
    if values.is_empty() {
        return Ok("list()".into());
    }
    let mut text = String::new();
    for (index, element) in values.iter().enumerate() {
        let label = element
            .name
            .as_ref()
            .map(|name| format!("{prefix}${name}"))
            .unwrap_or_else(|| format!("{prefix}[[{}]]", index + 1));
        let body = match &element.value {
            Sexp::List(nested) | Sexp::Dots(nested) => format_nested(nested, &label)?,
            value => format_sexp(value)?,
        };
        text.push_str(&format!("{label}\n{body}\n\n"));
    }
    text.pop();
    Ok(text)
}
