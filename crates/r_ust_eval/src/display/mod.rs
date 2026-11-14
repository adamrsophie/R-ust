use crate::{format_number, format_numeric_line, primitive_calls, EvalError, Sexp};
pub(crate) mod character;
mod chars;
mod closures;
mod containers;
mod lists;
mod scalars;
use containers::{format_data_frame, format_matrix};
use lists::format_list;
use scalars::format_factor;
/// Formats a value without the output event's final newline.
///
/// # Errors
///
/// Returns an error for unsupported language values or malformed canonical trees,
/// including failures nested inside containers. No output is emitted here.
pub fn format_sexp(value: &Sexp) -> Result<String, EvalError> {
    crate::ordinary_cycles::check(value, "cyclic ordinary value printing")?;
    Ok(match value {
        Sexp::ListObject(value) => format_sexp(&value.value()?)?,
        Sexp::PairListObject(value) => lists::format_pairlist(&value.elements()?)?,
        Sexp::Char(value) => chars::format(value)?,
        Sexp::MissingValue => String::new(),
        Sexp::PairList(values) => lists::format_pairlist(values.as_slice())?,
        Sexp::Environment(frame) => crate::environment_values::format(frame)?,
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
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => character::format(value, true)?,
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
        Sexp::ClassedList {
            elements, class, ..
        } => lists::format_classed(elements, class, "")?,
        Sexp::List(values) => format_list(values)?,
        Sexp::DataFrame { columns, nrow } => format_data_frame(columns, *nrow)?,
        Sexp::Matrix { data, nrow, ncol } => format_matrix(data, *nrow, *ncol)?,
        Sexp::Dots(values) => format_list(values)?,
        Sexp::Promise { .. } => "<promise>".to_string(),
        Sexp::Symbol(_) | Sexp::Call { .. } => {
            return crate::language_print::format_language(value).map_err(EvalError::from)
        }
        Sexp::Closure {
            parameters,
            body,
            environment,
        } => closures::format(parameters, body, environment)?,
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
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::InternalCall { .. }
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
            return Err(EvalError::NotImplemented("optimized language display"))
        }
    })
}
