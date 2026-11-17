use crate::{primitive_calls, EvalError, Sexp};
mod arrays;
mod attributed;
pub(crate) mod character;
mod chars;
mod closures;
mod containers;
mod dimensions;
mod labels;
mod lines;
mod list_arrays;
mod lists;
mod matrix;
mod metadata;
mod names;
mod numeric;
mod projection;
mod scalars;
mod vector;
use containers::{format_data_frame, format_matrix};
use lists::format_list;
use scalars::format_factor;

pub(crate) fn format_lines(value: &Sexp) -> Result<Option<String>, EvalError> {
    projection::check(value, &mut Vec::new())?;
    match value {
        Sexp::Closure { .. } => closures::format(value, true, ""),
        Sexp::Vector(values) => {
            if values.is_empty() {
                return Ok(Some(String::new()));
            }
            let mut output = None;
            for value in values {
                if let Some(value) = format_lines(value)? {
                    lines::append(&mut output, value);
                }
            }
            Ok(output)
        }
        _ => format_sexp(value).map(Some),
    }
}
/// Formats a value without the output event's final newline.
///
/// # Errors
///
/// Returns an error for unsupported language values or malformed canonical trees,
/// including failures nested inside containers. No output is emitted here.
pub fn format_sexp(value: &Sexp) -> Result<String, EvalError> {
    crate::character_attributes::validate(value)?;
    projection::check(value, &mut Vec::new())?;
    Ok(match value {
        Sexp::Managed(value) => format_sexp(value.borrow()?.value())?,
        Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. } => {
            attributed::render(value, true, "")?
        }
        Sexp::ListObject(value) => format_sexp(&value.value()?)?,
        Sexp::PairListObject(value) => lists::format_pairlist(&value.elements()?)?,
        Sexp::Char(value) => chars::format(value)?,
        Sexp::MissingValue => String::new(),
        Sexp::Expression { .. } => crate::language_print::format_language(value)?,
        Sexp::PairList(values) => lists::format_pairlist(values.as_slice())?,
        Sexp::Environment(frame) => crate::environment_values::format(frame)?,
        Sexp::Nil => "NULL".to_string(),
        Sexp::MissingArg(name) => format!("<missing {name}>"),
        Sexp::Primitive(id) => primitive_calls::format_primitive(*id),
        Sexp::Bytecode(code) => format!("<bytecode: {}>", code.display_address()?),
        Sexp::Raw(_) | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) => {
            vector::render(value, None, true)?
        }
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => character::format(value, true)?,
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => format_factor(codes, levels, *ordered),
        Sexp::Vector(_) => format_lines(value)?.unwrap_or_default(),
        Sexp::NamedList { .. } | Sexp::ClassedList { .. } => {
            let view = value
                .ordinary_list()
                .ok_or(EvalError::NotImplemented("ordinary list printing"))?;
            lists::format_ordinary(view, "")?
        }
        Sexp::List(values) => format_list(values)?,
        Sexp::DataFrame { columns, nrow } => format_data_frame(columns, *nrow)?,
        Sexp::Matrix { data, nrow, ncol } => format_matrix(data, *nrow, *ncol)?,
        Sexp::Dots(values) => format_list(values)?,
        Sexp::Promise { .. } => "<promise>".to_string(),
        Sexp::Symbol(_) | Sexp::Call { .. } => {
            return crate::language_print::format_language(value).map_err(EvalError::from)
        }
        Sexp::Closure { .. } => closures::format(value, true, "")?.unwrap_or_default(),
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod correction_tests;
