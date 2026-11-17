use super::format_lines;
use crate::{EvalError, ListElement, Sexp};

pub(super) fn format_list(values: &[ListElement]) -> Result<String, EvalError> {
    format_nested(
        values,
        "",
        (false, values.iter().any(|value| value.name.is_some())),
    )
}

pub(super) fn format_pairlist(values: &[ListElement]) -> Result<String, EvalError> {
    format_nested(values, "", (true, false))
}

fn format_nested(
    values: &[ListElement],
    prefix: &str,
    (pairlist, names_present): (bool, bool),
) -> Result<String, EvalError> {
    if values.is_empty() {
        return Ok(if names_present {
            "named list()"
        } else {
            "list()"
        }
        .into());
    }
    let mut text = String::new();
    for (index, element) in values.iter().enumerate() {
        let label = match &element.name {
            Some(name) if pairlist => format!("{prefix}${}", pairlist_name(name)?),
            Some(name) if !name.is_empty() => format!("{prefix}${}", super::labels::name(name)),
            None if names_present => format!("{prefix}$<NA>"),
            _ => format!("{prefix}[[{}]]", index + 1),
        };
        let projection = super::projection::read(&element.value)?;
        let body = if let Some(view) = projection.ordinary_list() {
            Some(format_ordinary(view, &label)?)
        } else {
            match projection.as_ref() {
                Sexp::AttributedAggregate { .. } | Sexp::AttributedAtomic { .. } => {
                    super::character::render(projection.as_ref(), true, &label)?
                }
                Sexp::List(nested) | Sexp::Dots(nested) => Some(format_nested(
                    nested,
                    &label,
                    (false, nested.iter().any(|value| value.name.is_some())),
                )?),
                Sexp::ClassedList {
                    elements, class, ..
                } => Some(format_classed(elements, class, &label)?),
                Sexp::PairList(nested) => {
                    Some(format_nested(nested.as_slice(), &label, (true, false))?)
                }
                value => format_lines(value)?,
            }
        };
        text.push_str(&label);
        text.push('\n');
        if let Some(body) = body {
            text.push_str(&body);
            text.push('\n');
        }
        text.push('\n');
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
        format_nested(
            values,
            "",
            (false, values.iter().any(|value| value.name.is_some()))
        )?,
        super::character::render(class, true, "attr(,\"class\")")?.unwrap_or_default()
    ))
}

pub(super) fn format_ordinary(
    view: r_ust_sexp::OrdinaryListView<'_>,
    prefix: &str,
) -> Result<String, EvalError> {
    let prefix = if view.class.is_some() { "" } else { prefix };
    let mut output = format_nested(view.elements, prefix, (false, view.names_present))?;
    if let Some(class) = view.class {
        crate::character_attributes::validate_class(class)?;
        output.push_str(&format!(
            "\nattr(,\"class\")\n{}",
            super::character::render(class, true, "attr(,\"class\")")?.unwrap_or_default()
        ));
    }
    Ok(output)
}
