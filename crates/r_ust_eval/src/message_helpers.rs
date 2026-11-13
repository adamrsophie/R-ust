use super::{string_values, CallArgument, EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_dots_length(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        self.eval_arguments(arguments, false)?;
        if !arguments.is_empty() {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to '...length' which requires 0",
                arguments.len()
            )));
        }
        let Some(Sexp::Dots(values)) = self.raw_lookup("...")? else {
            return Err(EvalError::Domain(
                "incorrect context: the current call has no '...' to look in".into(),
            ));
        };
        let length = i32::try_from(values.len())
            .map_err(|_| EvalError::Domain("dots length overflow".into()))?;
        Ok(Sexp::Integer(vec![Some(length)]))
    }

    pub(super) fn lookup_dot(&mut self, index: usize) -> Result<Sexp, EvalError> {
        if index == 0 {
            return Err(EvalError::Domain(
                "indexing '...' with non-positive index 0".into(),
            ));
        }
        let Some(Sexp::Dots(values)) = self.raw_lookup("...")? else {
            return Err(EvalError::Domain(format!(
                "..{index} used in an incorrect context, no ... to look in"
            )));
        };
        let value = values
            .get(index - 1)
            .ok_or_else(|| {
                let suffix = if index == 1 { "" } else { "s" };
                EvalError::Domain(format!(
                    "the ... list contains fewer than {index} element{suffix}"
                ))
            })?
            .value
            .clone();
        if matches!(value, Sexp::MissingArg(_)) {
            return Err(EvalError::MissingArgument(format!("..{index}")));
        }
        self.force_value(value)
    }
}

pub(super) fn list_character(value: &Sexp) -> Result<String, EvalError> {
    match value {
        Sexp::String(values) if values.len() == 1 => {
            Ok(values[0].clone().unwrap_or_else(|| "NA".into()))
        }
        Sexp::Logical(values) if values.len() == 1 => Ok(match values[0] {
            Some(true) => "TRUE",
            Some(false) => "FALSE",
            None => "NA",
        }
        .into()),
        Sexp::Integer(values) if values.len() == 1 => {
            Ok(values[0].map_or_else(|| "NA".into(), |value| value.to_string()))
        }
        Sexp::Real(values) if values.len() == 1 => Ok(super::format_number(values[0])),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
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
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => deparse_value(value),
    }
}

fn deparse_value(value: &Sexp) -> Result<String, EvalError> {
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
        Sexp::List(values) => {
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
        Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
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
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
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
