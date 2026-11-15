use crate::{character_attributes, EvalError, ListElement, Sexp};

pub(crate) fn format(value: &Sexp, quote: bool) -> Result<String, EvalError> {
    Ok(format_lines(value, quote)?.unwrap_or_default())
}

pub(crate) fn format_lines(value: &Sexp, quote: bool) -> Result<Option<String>, EvalError> {
    crate::ordinary_cycles::check(value, "cyclic ordinary value printing")?;
    render(value, quote, "")
}

pub(super) fn render(value: &Sexp, quote: bool, prefix: &str) -> Result<Option<String>, EvalError> {
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Some(values) = value.character_values() {
        let attributes = character_attributes::attributes(value)?;
        let mut output = Some(data(values, quote));
        for attribute in attributes {
            let name = attribute
                .name
                .as_deref()
                .ok_or_else(|| EvalError::Type("unnamed character attribute".into()))?;
            if matches!(name, "srcref" | "wholeSrcref" | "srcfile") {
                continue;
            }
            let label = format!(
                "{prefix}attr(,\"{}\")",
                crate::encode_string::encode(name, None)
            );
            super::lines::append(&mut output, label.clone());
            if let Some(value) = render(&attribute.value, quote, &label)? {
                super::lines::append(&mut output, value);
            }
        }
        return Ok(output);
    }
    if let Some(view) = value.ordinary_list() {
        let prefix = if view.class.is_some() { "" } else { prefix };
        let mut output = Some(list(view.elements, quote, (prefix, view.names_present))?);
        if let Some(class) = view.class {
            character_attributes::validate_class(class)?;
            let label = "attr(,\"class\")";
            super::lines::append(&mut output, label.into());
            if let Some(value) = render(class, quote, label)? {
                super::lines::append(&mut output, value);
            }
        }
        return Ok(output);
    }
    match value {
        Sexp::Dots(elements) => list(
            elements,
            quote,
            (
                prefix,
                elements.iter().any(|element| element.name.is_some()),
            ),
        )
        .map(Some),
        Sexp::Closure { .. } => super::closures::format(value, quote, prefix),
        _ => super::format_lines(value),
    }
}

fn list(
    elements: &[ListElement],
    quote: bool,
    (prefix, names_present): (&str, bool),
) -> Result<String, EvalError> {
    if elements.is_empty() {
        return Ok(if names_present {
            "named list()"
        } else {
            "list()"
        }
        .into());
    }
    let mut output = String::new();
    for (index, element) in elements.iter().enumerate() {
        let label = match &element.name {
            Some(name) if !name.is_empty() => format!("{prefix}${name}"),
            None if names_present => format!("{prefix}$<NA>"),
            _ => format!("{prefix}[[{}]]", index + 1),
        };
        let projection = crate::ordinary_values::read(&element.value)?;
        let child_prefix = if projection.ordinary_list().is_some()
            || matches!(projection.as_ref(), Sexp::Dots(_))
        {
            &label
        } else {
            ""
        };
        output.push_str(&label);
        output.push('\n');
        if let Some(body) = render(projection.as_ref(), quote, child_prefix)? {
            output.push_str(&body);
            output.push('\n');
        }
        output.push('\n');
    }
    output.pop();
    Ok(output)
}

fn data(values: &[Option<String>], quote: bool) -> String {
    if values.is_empty() {
        return "character(0)".into();
    }
    let values: Vec<_> = values
        .iter()
        .map(|value| match value {
            Some(value) if quote => crate::encode_string::encode(value, Some('"')),
            Some(value) => crate::encode_string::encode(value, None),
            None if quote => "NA".into(),
            None => "<NA>".into(),
        })
        .collect();
    let field = values
        .iter()
        .map(|value| crate::encode_string::width(value))
        .max()
        .unwrap_or(0);
    let label_width = values.len().to_string().len() + 2;
    let mut output = format!("{:>label_width$}", "[1]");
    let mut width = label_width;
    for (index, value) in values.iter().enumerate() {
        // shortcut: uses the default 80-column profile; thread print options here when general print controls are implemented.
        if index > 0 && width + field + 1 > 80 {
            output.push_str(&format!("\n{:>label_width$}", format!("[{}]", index + 1)));
            width = label_width;
        }
        output.push(' ');
        output.push_str(value);
        output.push_str(&" ".repeat(field.saturating_sub(crate::encode_string::width(value))));
        width += field + 1;
    }
    output
}
