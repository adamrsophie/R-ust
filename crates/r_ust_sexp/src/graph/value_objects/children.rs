use super::super::GraphError;
use crate::Sexp;

// Visits only immediate children. Plans replace each with a typed graph slot.
pub(super) fn children(
    value: &mut Sexp,
    map: &mut impl FnMut(&mut Sexp) -> Result<(), GraphError>,
) -> Result<(), GraphError> {
    match value {
        Sexp::Nil
        | Sexp::Primitive(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Factor { .. }
        | Sexp::Break
        | Sexp::Next => {}
        Sexp::Char(value) => {
            if value.as_ref().is_some_and(|value| value.contains('\0')) {
                return Err(GraphError::InvalidCharacter);
            }
        }
        Sexp::Bytecode(_)
        | Sexp::Environment(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Promise { .. } => {}
        Sexp::Closure { .. } => return Err(GraphError::WrongKind),
        Sexp::Function { parameters, body } => {
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    map(default)?;
                }
            }
            map(body)?;
        }
        Sexp::Vector(values) | Sexp::Block(values) => {
            for value in values {
                map(value)?;
            }
        }
        Sexp::PairList(values) => {
            for value in values.elements_mut() {
                map(&mut value.value)?;
            }
        }
        Sexp::NamedList {
            elements, class, ..
        } => {
            if let Some(class) = class {
                map(class)?;
            }
            for element in elements {
                map(&mut element.value)?;
            }
        }
        Sexp::ClassedList {
            elements, class, ..
        } => {
            map(class)?;
            for element in elements {
                map(&mut element.value)?;
            }
        }
        Sexp::AttributedCharacter {
            attributes: values, ..
        }
        | Sexp::Expression {
            elements: values, ..
        }
        | Sexp::List(values)
        | Sexp::Dots(values)
        | Sexp::DataFrame {
            columns: values, ..
        } => {
            for value in values {
                map(&mut value.value)?;
            }
        }
        Sexp::Matrix { data: value, .. }
        | Sexp::Repeat { body: value }
        | Sexp::Dollar { value, .. }
        | Sexp::Assignment { value, .. }
        | Sexp::SuperAssignment { value, .. }
        | Sexp::DollarAssignment { value, .. }
        | Sexp::NamesAssignment { value, .. }
        | Sexp::Unary { value, .. }
        | Sexp::Parenthesized(value) => map(value)?,
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            map(condition)?;
            map(then_branch)?;
            if let Some(value) = else_branch {
                map(value)?;
            }
        }
        Sexp::For { sequence, body, .. }
        | Sexp::While {
            condition: sequence,
            body,
        } => {
            map(sequence)?;
            map(body)?;
        }
        Sexp::Subset { value, index } | Sexp::Subassignment { value, index, .. } => {
            map(value)?;
            if let Some(value) = index {
                map(value)?;
            }
        }
        Sexp::Subset2D { value, row, col }
        | Sexp::MatrixSubassignment {
            value, row, col, ..
        } => {
            map(value)?;
            if let Some(value) = row {
                map(value)?;
            }
            if let Some(value) = col {
                map(value)?;
            }
        }
        Sexp::Extract { value, index } | Sexp::ExtractAssignment { value, index, .. } => {
            map(value)?;
            map(index)?;
        }
        Sexp::ReplacementAssignment { target, value, .. }
        | Sexp::MemberAssignment { target, value, .. } => {
            map(target)?;
            map(value)?;
        }
        Sexp::ExtractArguments {
            value: function,
            arguments,
        }
        | Sexp::Call {
            function,
            arguments,
        } => {
            map(function)?;
            for argument in arguments {
                map(&mut argument.value)?;
            }
        }
        Sexp::InternalCall { arguments, .. } => {
            for argument in arguments {
                map(&mut argument.value)?;
            }
        }
        Sexp::Binary { left, right, .. } => {
            map(left)?;
            map(right)?;
        }
    }
    Ok(())
}
