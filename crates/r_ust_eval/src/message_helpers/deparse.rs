use crate::{string_values, EvalError, Sexp};

pub(super) fn deparse_value(value: &Sexp) -> Result<String, EvalError> {
    crate::ordinary_cycles::check(value, "cyclic ordinary value coercion")?;
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    let (empty, parts) = match value {
        Sexp::Nil => return Ok("NULL".into()),
        Sexp::String(values) => (
            "character(0)",
            values
                .iter()
                .map(|value| {
                    value
                        .as_ref()
                        .map_or_else(|| "NA".into(), |value| quote(value))
                })
                .collect::<Vec<_>>(),
        ),
        Sexp::Logical(_) => (
            "logical(0)",
            string_values(value)?
                .into_iter()
                .map(|value| value.unwrap_or_else(|| "NA".into()))
                .collect(),
        ),
        Sexp::Integer(values) => {
            if let [Some(first), Some(second), ..] = values.as_slice() {
                let step = second.checked_sub(*first);
                if matches!(step, Some(1 | -1))
                    && values.windows(2).all(|pair| match pair {
                        [Some(left), Some(right)] => right.checked_sub(*left) == step,
                        _ => false,
                    })
                {
                    if let Some(Some(last)) = values.last() {
                        return Ok(format!("{first}:{last}"));
                    }
                }
            }
            (
                "integer(0)",
                string_values(value)?
                    .into_iter()
                    .map(|value| value.unwrap_or_else(|| "NA".into()))
                    .collect(),
            )
        }
        Sexp::Real(_) => (
            "numeric(0)",
            string_values(value)?
                .into_iter()
                .map(|value| value.unwrap_or_else(|| "NA".into()))
                .collect(),
        ),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let parts = values
                .iter()
                .map(|element| {
                    let value = deparse_value(&element.value)?;
                    Ok(match &element.name {
                        Some(name) => format!("{name} = {value}"),
                        None => value,
                    })
                })
                .collect::<Result<Vec<_>, EvalError>>()?;
            return Ok(format!("list({})", parts.join(", ")));
        }
        Sexp::Vector(values) => {
            return Ok(format!(
                "list({})",
                values
                    .iter()
                    .map(deparse_value)
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ))
        }
        Sexp::Matrix { data, .. } => return deparse_value(data),
        Sexp::Factor { codes, .. } => {
            let values = codes
                .iter()
                .map(|value| {
                    value
                        .map(i32::try_from)
                        .transpose()
                        .map_err(|_| EvalError::Domain("factor code overflow".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            return deparse_value(&Sexp::Integer(values));
        }
        Sexp::AttributedCharacter { .. }
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Char(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Block(_)
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::ReplacementAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => {
            return Err(EvalError::NotImplemented(
                "list element character deparsing",
            ))
        }
    };
    Ok(match parts.as_slice() {
        [] => empty.into(),
        [single] => single.clone(),
        _ => format!("c({})", parts.join(", ")),
    })
}

fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character => out.push(character),
        }
    }
    out.push('"');
    out
}
