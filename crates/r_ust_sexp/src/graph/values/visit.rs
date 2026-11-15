use super::*;
use crate::ClosureEnvironment;

// One exhaustive walk owns link conversion and exact real-payload inspection.
pub(super) fn visit(
    value: &mut Sexp,
    map: &mut impl FnMut(&mut Link) -> Result<(), GraphError>,
) -> Result<(), GraphError> {
    walk(value, map, &mut |_| {})
}

pub(super) fn real_bits(value: &mut Sexp) -> Result<Vec<u64>, GraphError> {
    let mut bits = Vec::new();
    walk(value, &mut |_| Ok(()), &mut |values| {
        for value in values {
            bits.push(value.to_bits());
            *value = 0.0;
        }
    })?;
    Ok(bits)
}

fn walk(
    value: &mut Sexp,
    map: &mut impl FnMut(&mut Link) -> Result<(), GraphError>,
    reals: &mut impl FnMut(&mut [f64]),
) -> Result<(), GraphError> {
    match value {
        Sexp::Nil
        | Sexp::Primitive(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::String(_)
        | Sexp::Factor { .. }
        | Sexp::Break
        | Sexp::Next => {}
        Sexp::Real(values) => reals(values),
        Sexp::Char(value) => {
            if value.as_ref().is_some_and(|value| value.contains('\0')) {
                return Err(GraphError::InvalidCharacter);
            }
        }
        Sexp::Environment(frame) => map(&mut frame.0)?,
        Sexp::Bytecode(code) => map(&mut code.0)?,
        Sexp::ListObject(value) => map(&mut value.0)?,
        Sexp::PairListObject(value) => map(&mut value.0)?,
        Sexp::Promise { state } => map(&mut state.0)?,
        Sexp::Closure {
            parameters,
            body,
            environment,
            attributes,
        } => {
            match environment {
                ClosureEnvironment::Captured(frame) => map(&mut frame.0)?,
                ClosureEnvironment::Base => {}
            }
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    walk(default, map, reals)?;
                }
            }
            walk(body, map, reals)?;
            for attribute in attributes {
                walk(&mut attribute.value, map, reals)?;
            }
        }
        Sexp::Function { parameters, body } => {
            for parameter in parameters {
                if let Some(default) = &mut parameter.default {
                    walk(default, map, reals)?;
                }
            }
            walk(body, map, reals)?;
        }
        Sexp::Vector(values) | Sexp::Block(values) => {
            for value in values {
                walk(value, map, reals)?;
            }
        }
        Sexp::PairList(values) => {
            for value in values.elements_mut() {
                walk(&mut value.value, map, reals)?;
            }
        }
        Sexp::NamedList {
            elements, class, ..
        } => {
            if let Some(class) = class {
                walk(class, map, reals)?;
            }
            for element in elements {
                walk(&mut element.value, map, reals)?;
            }
        }
        Sexp::ClassedList {
            elements, class, ..
        } => {
            walk(class, map, reals)?;
            for element in elements {
                walk(&mut element.value, map, reals)?;
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
                walk(&mut value.value, map, reals)?;
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
        | Sexp::Parenthesized(value) => walk(value, map, reals)?,
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            walk(condition, map, reals)?;
            walk(then_branch, map, reals)?;
            if let Some(value) = else_branch {
                walk(value, map, reals)?;
            }
        }
        Sexp::For { sequence, body, .. }
        | Sexp::While {
            condition: sequence,
            body,
        } => {
            walk(sequence, map, reals)?;
            walk(body, map, reals)?;
        }
        Sexp::Subset { value, index } | Sexp::Subassignment { value, index, .. } => {
            walk(value, map, reals)?;
            if let Some(value) = index {
                walk(value, map, reals)?;
            }
        }
        Sexp::Subset2D { value, row, col }
        | Sexp::MatrixSubassignment {
            value, row, col, ..
        } => {
            walk(value, map, reals)?;
            if let Some(value) = row {
                walk(value, map, reals)?;
            }
            if let Some(value) = col {
                walk(value, map, reals)?;
            }
        }
        Sexp::Extract { value, index } | Sexp::ExtractAssignment { value, index, .. } => {
            walk(value, map, reals)?;
            walk(index, map, reals)?;
        }
        Sexp::ReplacementAssignment { target, value, .. }
        | Sexp::MemberAssignment { target, value, .. } => {
            walk(target, map, reals)?;
            walk(value, map, reals)?;
        }
        Sexp::ExtractArguments {
            value: function,
            arguments,
        }
        | Sexp::Call {
            function,
            arguments,
        } => {
            walk(function, map, reals)?;
            for argument in arguments {
                walk(&mut argument.value, map, reals)?;
            }
        }
        Sexp::InternalCall { arguments, .. } => {
            for argument in arguments {
                walk(&mut argument.value, map, reals)?;
            }
        }
        Sexp::Binary { left, right, .. } => {
            walk(left, map, reals)?;
            walk(right, map, reals)?;
        }
    }
    Ok(())
}
