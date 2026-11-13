mod constants;
mod definitions;
mod encode_string;
mod evaluation;
mod rowsum;
mod run_length;
mod sequence;

use std::sync::OnceLock;

use r_ust_sexp::ClosureEnvironment;

use super::{EvalError, Parser, Sexp};

const SEQUENCE_DEFINITIONS: &[(&str, &str)] = &[
    ("sequence", sequence::GENERIC),
    ("sequence.default", sequence::DEFAULT),
];

fn definitions() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    definitions::DEFINITIONS
        .iter()
        .chain(run_length::DEFINITIONS.iter())
        .chain(rowsum::DEFINITIONS.iter())
        .chain(encode_string::DEFINITIONS.iter())
        .chain(evaluation::DEFINITIONS.iter())
        .chain(SEQUENCE_DEFINITIONS.iter())
}

pub(super) fn lookup(name: &str) -> Result<Option<Sexp>, EvalError> {
    static BASE: OnceLock<Result<Vec<(String, Sexp)>, EvalError>> = OnceLock::new();
    if let Some(value) = constants::lookup(name) {
        return Ok(Some(value));
    }
    if !definitions().any(|(key, _)| *key == name) {
        return Ok(None);
    }
    let registry = BASE.get_or_init(|| {
        definitions()
            .map(|(name, source)| {
                let parsed = Parser.parse_all(source).map_err(|error| {
                    EvalError::Parse(format!("base bootstrap: {}", error.message))
                })?;
                let [Sexp::Function { parameters, body }] = parsed.expressions.as_slice() else {
                    return Err(EvalError::NotImplemented("base closure bootstrap shape"));
                };
                let mut body = body.clone();
                if *name == "sequence" {
                    sequence::lower_generic(&mut body);
                }
                if let Some(kernel) = run_length::lower(name) {
                    *body = kernel;
                } else if *name == "encodeString" {
                    *body = Sexp::InternalCall {
                        operation: r_ust_sexp::InternalOperation::EncodeStringWrapper,
                        arguments: Vec::new(),
                    };
                } else {
                    lower(&mut body)?;
                }
                Ok((
                    (*name).into(),
                    Sexp::Closure {
                        parameters: parameters.clone(),
                        body,
                        environment: ClosureEnvironment::Base,
                    },
                ))
            })
            .collect()
    });
    match registry {
        Ok(values) => Ok(values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())),
        Err(error) => Err(error.clone()),
    }
}

fn lower(value: &mut Sexp) -> Result<(), EvalError> {
    match value {
        Sexp::Call {
            function,
            arguments,
        } => {
            if let Sexp::Symbol(name) = function.as_ref() {
                if let Some(operation) = rowsum::resolve(name) {
                    for argument in arguments.iter_mut() {
                        lower(&mut argument.value)?;
                    }
                    *value = Sexp::InternalCall {
                        operation,
                        arguments: std::mem::take(arguments),
                    };
                    return Ok(());
                }
            }
            if matches!(function.as_ref(), Sexp::Symbol(name) if name == ".Internal") {
                let [argument] = arguments.as_mut_slice() else {
                    return Err(EvalError::Arity("invalid base .Internal call".into()));
                };
                let Sexp::Call {
                    function,
                    arguments,
                } = &mut argument.value
                else {
                    return Err(EvalError::Type("invalid base .Internal argument".into()));
                };
                let Sexp::Symbol(name) = function.as_ref() else {
                    return Err(EvalError::Type("invalid base internal identity".into()));
                };
                let operation = super::internal::resolve(name)
                    .ok_or(EvalError::NotImplemented("unresolved R internal"))?;
                for argument in arguments.iter_mut() {
                    lower(&mut argument.value)?;
                }
                *value = Sexp::InternalCall {
                    operation,
                    arguments: std::mem::take(arguments),
                };
            } else {
                lower(function)?;
                for argument in arguments {
                    lower(&mut argument.value)?;
                }
            }
        }
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            lower(condition)?;
            lower(then_branch)?;
            if let Some(branch) = else_branch {
                lower(branch)?;
            }
        }
        Sexp::Block(values) => {
            for value in values {
                lower(value)?;
            }
        }
        Sexp::Assignment { value, .. } | Sexp::Unary { value, .. } | Sexp::Parenthesized(value) => {
            lower(value)?
        }
        Sexp::Binary { left, right, .. } => {
            lower(left)?;
            lower(right)?;
        }
        Sexp::Nil | Sexp::Symbol(_) | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::String(_) => {}
        Sexp::MissingArg(_)
        | Sexp::Real(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. } => {
            return Err(EvalError::NotImplemented("base bootstrap expression"))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_internal_rejects_unresolved_identities() {
        let mut expression = Parser
            .parse_all(".Internal(unknown_internal())")
            .unwrap()
            .expressions
            .remove(0);
        assert_eq!(
            lower(&mut expression),
            Err(EvalError::NotImplemented("unresolved R internal"))
        );
    }
}
