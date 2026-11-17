use crate::{EvalError, ListElement, Sexp};

pub(super) fn render(value: &Sexp, quote: bool, prefix: &str) -> Result<String, EvalError> {
    let (mut output, attributes) = match value {
        Sexp::AttributedAtomic { values, attributes } => (
            super::arrays::render(&values.clone().into_value(), attributes, quote)?,
            attributes,
        ),
        Sexp::AttributedCharacter { values, attributes } => (
            super::arrays::render(&Sexp::String(values.clone()), attributes, quote)?,
            attributes,
        ),
        Sexp::AttributedAggregate { values, attributes } => {
            let elements = super::metadata::elements(values.values(), attributes)?;
            let names_present = super::names::read(attributes)?.is_some();
            let body = match values {
                r_ust_sexp::AggregateVector::Expression(_) => {
                    crate::language_print::format_language(&Sexp::Expression {
                        elements,
                        names_present,
                    })?
                }
                r_ust_sexp::AggregateVector::Vector(_) => {
                    if super::dimensions::read(attributes)?.is_some_and(|d| d.sizes.len() > 1) {
                        super::list_arrays::render(values.values(), attributes)?
                    } else {
                        super::character::list(&elements, quote, (prefix, names_present))?
                    }
                }
            };
            (body, attributes)
        }
        _ => return Err(EvalError::Type("expected attributed print value".into())),
    };
    append(&mut output, attributes, (quote, prefix))?;
    Ok(output)
}

fn append(
    output: &mut String,
    attributes: &[ListElement],
    (quote, prefix): (bool, &str),
) -> Result<(), EvalError> {
    let array = super::metadata::get(attributes, "dim")?.is_some();
    let class = super::metadata::get(attributes, "class")?
        .map(|v| super::metadata::strings(&v))
        .transpose()?
        .unwrap_or_default();
    let inherits = |name| class.iter().any(|v| v.as_deref() == Some(name));
    let prefix = if prefix.ends_with(')') { prefix } else { "" };
    for attribute in attributes {
        let name = attribute
            .name
            .as_deref()
            .ok_or_else(|| EvalError::Type("unnamed print attribute".into()))?;
        if matches!(name, "comment" | "srcref" | "wholeSrcref" | "srcfile")
            || (array && matches!(name, "dim" | "dimnames"))
            || (!array && name == "names")
            || (inherits("factor") && matches!(name, "levels" | "class"))
            || (inherits("data.frame") && name == "row.names")
        {
            continue;
        }
        let label = format!(
            "{prefix}attr(,\"{}\")",
            crate::encode_string::encode(name, None)
        );
        output.push('\n');
        output.push_str(&label);
        let value = super::projection::read(&attribute.value)?;
        let rows;
        let value = if name == "row.names" {
            rows = r_ust_sexp::expanded_row_names(value.as_ref())?;
            &rows
        } else {
            value.as_ref()
        };
        if let Some(body) = super::character::render(value, quote, &label)? {
            output.push('\n');
            output.push_str(&body);
        }
    }
    Ok(())
}
