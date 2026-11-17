use crate::{EvalError, ListElement, Sexp};

pub(super) fn render(values: &[Sexp], attributes: &[ListElement]) -> Result<String, EvalError> {
    let summaries = values
        .iter()
        .map(|v| summary(v).map(Some))
        .collect::<Result<Vec<_>, _>>()?;
    super::arrays::render(&Sexp::String(summaries), attributes, false)
}

fn factor_summary(value: &Sexp) -> Result<Option<String>, EvalError> {
    let Some(Sexp::AttributedAtomic {
        values: r_ust_sexp::AtomicVector::Integer(codes),
        attributes,
    }) = value.attribute_description()?
    else {
        return Ok(None);
    };
    let Some(class) = super::metadata::get(&attributes, "class")? else {
        return Ok(None);
    };
    let classes = super::metadata::strings(&class)?;
    if !classes.iter().any(|c| c.as_deref() == Some("factor")) {
        return Ok(None);
    }
    if !matches!(
        classes.first().and_then(|c| c.as_deref()),
        Some("factor" | "ordered")
    ) {
        return Err(EvalError::NotImplemented(
            "list array custom factor format dispatch",
        ));
    }
    if codes.len() != 1 {
        return Ok(Some(format!(
            "{},{}",
            classes[0].as_deref().unwrap_or("factor"),
            codes.len()
        )));
    }
    let Some(code) = codes[0] else {
        return Ok(Some("NA".into()));
    };
    let levels = super::metadata::get(&attributes, "levels")?
        .ok_or_else(|| EvalError::Type("malformed factor".into()))?;
    let levels = super::metadata::strings(&levels)?;
    let index = usize::try_from(code)
        .ok()
        .and_then(|i| i.checked_sub(1))
        .ok_or_else(|| EvalError::Type("malformed factor".into()))?;
    let label = levels
        .get(index)
        .ok_or_else(|| EvalError::Type("malformed factor".into()))?;
    Ok(Some(label.clone().unwrap_or_else(|| "NA".into())))
}

fn summary(value: &Sexp) -> Result<String, EvalError> {
    let value = super::projection::read(value)?;
    if let Some(summary) = factor_summary(value.as_ref())? {
        return Ok(summary);
    }
    match value.as_ref() {
        Sexp::AttributedAtomic { values, attributes } => {
            if super::metadata::get(attributes, "class")?.is_some() {
                return Err(EvalError::NotImplemented(
                    "list array class format dispatch",
                ));
            }
            return summary(&values.clone().into_value());
        }
        Sexp::AttributedAggregate { values, attributes } => {
            if super::metadata::get(attributes, "class")?.is_some() {
                return Err(EvalError::NotImplemented(
                    "list array class format dispatch",
                ));
            }
            return Ok(match values {
                r_ust_sexp::AggregateVector::Vector(values) => format!("list,{}", values.len()),
                r_ust_sexp::AggregateVector::Expression(_) => "?".into(),
            });
        }
        Sexp::AttributedCharacter { values, attributes } => {
            if super::metadata::get(attributes, "class")?.is_some() {
                return Err(EvalError::NotImplemented(
                    "list array class format dispatch",
                ));
            }
            return summary(&Sexp::String(values.clone()));
        }
        Sexp::Factor { .. } | Sexp::ClassedList { .. } | Sexp::DataFrame { .. } => {
            return Err(EvalError::NotImplemented(
                "list array class format dispatch",
            ));
        }
        Sexp::Closure { attributes, .. }
            if super::metadata::get(attributes, "class")?.is_some() =>
        {
            return Err(EvalError::NotImplemented(
                "list array class format dispatch",
            ));
        }
        Sexp::Matrix { data, .. } => return summary(data),
        Sexp::String(values) if values.len() == 1 => {
            let text = values[0].as_deref().unwrap_or("NA");
            if text.len() < 100 {
                return Ok(format!("\"{text}\""));
            }
            let end = text
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|i| *i <= 99)
                .last()
                .unwrap_or(0);
            return Ok(format!("\"{}\" [truncated]", &text[..end]));
        }
        Sexp::Nil => return Ok("NULL".into()),
        Sexp::Call { .. } => return Ok("expression".into()),
        Sexp::List(values)
        | Sexp::NamedList {
            elements: values, ..
        } => return Ok(format!("list,{}", values.len())),
        Sexp::PairList(values) => return Ok(format!("list,{}", values.as_slice().len())),
        Sexp::Vector(values) => return Ok(format!("list,{}", values.len())),
        Sexp::Raw(_) | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) => {}
        _ => return Ok("?".into()),
    }
    let (tokens, _, empty) = super::numeric::tokens(value.as_ref(), true)?;
    if tokens.len() == 1 {
        Ok(tokens[0].clone())
    } else {
        Ok(format!(
            "{},{}",
            empty.trim_end_matches("(0)"),
            tokens.len()
        ))
    }
}
