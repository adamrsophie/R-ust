use super::format_sexp;
use crate::{EvalError, ListElement, Sexp};

pub(super) fn format_list(values: &[ListElement]) -> Result<String, EvalError> {
    format_nested(values, "", false)
}

pub(super) fn format_pairlist(values: &[ListElement]) -> Result<String, EvalError> {
    format_nested(values, "", true)
}

fn format_nested(
    values: &[ListElement],
    prefix: &str,
    pairlist: bool,
) -> Result<String, EvalError> {
    if values.is_empty() {
        return Ok("list()".into());
    }
    let mut text = String::new();
    for (index, element) in values.iter().enumerate() {
        let label = match &element.name {
            Some(name) if pairlist => format!("{prefix}${}", pairlist_name(name)?),
            Some(name) => format!("{prefix}${name}"),
            None => format!("{prefix}[[{}]]", index + 1),
        };
        let projection = crate::ordinary_values::read(&element.value)?;
        let body = match projection.as_ref() {
            Sexp::List(nested) | Sexp::Dots(nested) => format_nested(nested, &label, false)?,
            Sexp::ClassedList {
                elements, class, ..
            } => format_classed(elements, class, &label)?,
            Sexp::PairList(nested) => format_nested(nested.as_slice(), &label, true)?,
            value => format_sexp(value)?,
        };
        text.push_str(&format!("{label}\n{body}\n\n"));
    }
    text.pop();
    Ok(text)
}

fn pairlist_name(name: &str) -> Result<String, EvalError> {
    crate::language_print::check_text(name)?;
    Ok(if crate::language_print::valid_name(name) {
        name.to_owned()
    } else {
        format!("`{}`", crate::encode_string::encode(name, None))
    })
}

pub(super) fn format_classed(
    values: &[ListElement],
    class: &Sexp,
    _prefix: &str,
) -> Result<String, EvalError> {
    crate::character_attributes::validate_class(class)?;
    Ok(format!(
        "{}\nattr(,\"class\")\n{}",
        format_nested(values, "", false)?,
        super::character::render(class, true, "attr(,\"class\")")?
    ))
}
