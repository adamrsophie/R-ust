use super::{canonical, promise_expression};
use crate::{CallArgument, EvalError, Sexp};

pub(super) fn render(arguments: &[&CallArgument]) -> Result<String, EvalError> {
    let values = arguments
        .iter()
        .map(|argument| {
            let value = promise_expression(&argument.value)?;
            Ok((argument.name.as_deref(), value))
        })
        .collect::<Result<Vec<_>, EvalError>>()?;
    let alist = values
        .iter()
        .any(|(_, value)| matches!(value, Sexp::MissingValue));
    let mut parts = Vec::new();
    for (name, value) in values {
        let value = crate::language_print::format_argument_syntax(&canonical(&value)?)?;
        parts.push(match name {
            Some(name) => {
                let name = crate::language_print::format_language(&Sexp::Symbol(name.into()))?;
                format!("{name} = {value}")
            }
            None => value,
        });
    }
    let contents = parts.join(", ");
    let contents = if alist {
        format!("(alist({contents}))")
    } else {
        format!("({contents})")
    };
    let suffix = if arguments.len() == 1 { "" } else { "s" };
    Ok(format!("unused argument{suffix} {contents}"))
}
