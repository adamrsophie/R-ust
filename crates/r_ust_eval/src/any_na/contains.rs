use super::*;

pub(super) fn contains(value: &Sexp, recursive: &Sexp) -> Result<bool, EvalError> {
    visit(value, recursive, &mut Vec::new())
}

fn visit(value: &Sexp, recursive: &Sexp, path: &mut Vec<Sexp>) -> Result<bool, EvalError> {
    let shared = matches!(
        value,
        Sexp::Managed(_) | Sexp::ListObject(_) | Sexp::PairListObject(_)
    );
    if shared {
        if path.contains(value) {
            return Err(EvalError::NotImplemented("cyclic ordinary value anyNA"));
        }
        path.push(value.clone());
    }
    let result = contents(value, recursive, path);
    if shared {
        path.pop();
    }
    result
}

fn contents(value: &Sexp, recursive: &Sexp, path: &mut Vec<Sexp>) -> Result<bool, EvalError> {
    if let Sexp::PairList(values) = value {
        return visit(&Sexp::List(values.as_slice().to_vec()), recursive, path);
    }
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        return visit(
            crate::ordinary_values::read(value)?.as_ref(),
            recursive,
            path,
        );
    }
    Ok(match value {
        Sexp::Managed(value) => visit(value.borrow()?.value(), recursive, path)?,
        Sexp::AttributedAtomic { values, .. } => {
            visit(&values.clone().into_value(), recursive, path)?
        }
        Sexp::AttributedAggregate {
            values: r_ust_sexp::AggregateVector::Vector(values),
            ..
        } => visit(&Sexp::Vector(values.clone()), recursive, path)?,
        Sexp::Nil | Sexp::Raw(_) => false,
        Sexp::Logical(values) => values.iter().any(Option::is_none),
        Sexp::Integer(values) => values.iter().any(Option::is_none),
        Sexp::Real(values) => values.iter().any(|value| value.is_nan()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.iter().any(Option::is_none)
        }
        Sexp::Factor { codes, .. } => codes.iter().any(Option::is_none),
        Sexp::Matrix { data, .. } => visit(data, recursive, path)?,
        Sexp::DataFrame { columns, .. } => {
            let mut missing = false;
            let default = Sexp::Logical(vec![Some(false)]);
            for column in columns {
                missing |= visit(&column.value, &default, path)?;
            }
            missing
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            let recurse = recursive_flag(recursive)?;
            for element in values {
                if if recurse {
                    visit(&element.value, recursive, path)?
                } else {
                    scalar_missing(&element.value)?
                } {
                    return Ok(true);
                }
            }
            false
        }
        Sexp::Vector(values) => {
            let recurse = recursive_flag(recursive)?;
            for element in values {
                if if recurse {
                    visit(element, recursive, path)?
                } else {
                    scalar_missing(element)?
                } {
                    return Ok(true);
                }
            }
            false
        }
        Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::Expression { .. }
        | Sexp::AttributedAggregate {
            values: r_ust_sexp::AggregateVector::Expression(_),
            ..
        }
        | Sexp::PairList(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Char(_)
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Bytecode(_)
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
            return Err(EvalError::Type(format!(
                "anyNA() applied to non-(list or vector) of type '{}'",
                typeof_value(value)
            )))
        }
    })
}
