use crate::{format_line, format_number, format_numeric_line, primitive_calls, EvalError, Sexp};
mod containers;
mod scalars;
use containers::{format_data_frame, format_list, format_matrix};
use scalars::{format_factor, format_function_signature};
/// Formats a value without the output event's final newline.
///
/// # Errors
///
/// Returns an error for unsupported language values or malformed canonical trees,
/// including failures nested inside containers. No output is emitted here.
pub fn format_sexp(value: &Sexp) -> Result<String, EvalError> {
    Ok(match value {
        Sexp::MissingValue => String::new(),
        Sexp::PairList(_) => return Err(EvalError::NotImplemented("pairlist display")),
        Sexp::Nil => "NULL".to_string(),
        Sexp::MissingArg(name) => format!("<missing {name}>"),
        Sexp::Primitive(id) => primitive_calls::format_primitive(*id),
        Sexp::Logical(values) => format_numeric_line(
            values
                .iter()
                .map(|value| match value {
                    Some(true) => "TRUE".to_string(),
                    Some(false) => "FALSE".to_string(),
                    None => "NA".to_string(),
                })
                .collect(),
            "logical(0)",
        ),
        Sexp::Integer(values) => format_numeric_line(
            values
                .iter()
                .map(|value| value.map_or_else(|| "NA".to_string(), |value| value.to_string()))
                .collect(),
            "integer(0)",
        ),
        Sexp::Real(values) => format_numeric_line(
            values.iter().map(|value| format_number(*value)).collect(),
            "numeric(0)",
        ),
        Sexp::String(values) => format_line(
            values
                .iter()
                .map(|value| match value {
                    Some(value) => format!("{value:?}"),
                    None => "NA".to_string(),
                })
                .collect(),
        ),
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => format_factor(codes, levels, *ordered),
        Sexp::Vector(values) => values
            .iter()
            .map(format_sexp)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n"),
        Sexp::List(values) => format_list(values)?,
        Sexp::DataFrame { columns, nrow } => format_data_frame(columns, *nrow)?,
        Sexp::Matrix { data, nrow, ncol } => format_matrix(data, *nrow, *ncol)?,
        Sexp::Dots(values) => format_list(values)?,
        Sexp::Promise { .. } => "<promise>".to_string(),
        Sexp::Symbol(_) | Sexp::Call { .. } => {
            return crate::language_print::format_language(value).map_err(EvalError::from)
        }
        Sexp::Closure { parameters, .. } => format_function_signature(parameters),
        Sexp::Function { .. }
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
        | Sexp::InternalCall { .. }
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
            return Err(EvalError::NotImplemented("optimized language display"))
        }
    })
}
