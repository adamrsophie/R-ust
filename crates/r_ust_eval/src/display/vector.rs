use crate::{EvalError, Sexp};

pub(super) fn render(
    value: &Sexp,
    names: Option<&[Option<String>]>,
    quote: bool,
) -> Result<String, EvalError> {
    let (values, character, empty) = super::numeric::tokens(value, quote)?;
    if values.is_empty() {
        return Ok(format!(
            "{}{empty}",
            if names.is_some() { "named " } else { "" }
        ));
    }
    if let Some(names) = names {
        return named(&values, names);
    }
    let field = values
        .iter()
        .map(|v| crate::encode_string::width(v))
        .max()
        .unwrap_or(0);
    let label = values.len().to_string().len() + 2;
    let count = ((80usize.saturating_sub(label)) / (field + 1)).max(1);
    let mut lines = Vec::new();
    for (chunk, values) in values.chunks(count).enumerate() {
        let mut line = format!("{:>label$}", format!("[{}]", chunk * count + 1));
        for value in values {
            line.push(' ');
            line.push_str(&pad(value, field, character));
        }
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn named(values: &[String], names: &[Option<String>]) -> Result<String, EvalError> {
    if values.len() != names.len() {
        return Err(EvalError::Type("invalid print names length".into()));
    }
    let names: Vec<_> = names
        .iter()
        .map(|n| {
            n.as_deref()
                .map(|n| crate::encode_string::encode(n, None))
                .unwrap_or_else(|| "<NA>".into())
        })
        .collect();
    let field = values
        .iter()
        .chain(&names)
        .map(|v| crate::encode_string::width(v))
        .max()
        .unwrap_or(0);
    let count = (80 / (field + 1)).max(1);
    let mut lines = Vec::new();
    for (values, names) in values.chunks(count).zip(names.chunks(count)) {
        for row in [names, values] {
            lines.push(
                row.iter()
                    .map(|v| format!("{} ", pad(v, field, false)))
                    .collect::<String>(),
            );
        }
    }
    Ok(lines.join("\n"))
}

pub(super) fn pad(value: &str, width: usize, left: bool) -> String {
    let padding = " ".repeat(width.saturating_sub(crate::encode_string::width(value)));
    if left {
        format!("{value}{padding}")
    } else {
        format!("{padding}{value}")
    }
}
