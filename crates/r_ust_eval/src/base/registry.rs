use std::sync::OnceLock;

use r_ust_sexp::{ClosureEnvironment, PrimitiveId};

use super::{
    bitwise, constants, definitions, encode_string, environments, evaluation, formals, list_values,
    lower, printing, rowsum, run_length, sequence, EvalError, Parser, Sexp,
};

const SEQUENCE_DEFINITIONS: &[(&str, &str)] = &[
    ("sequence", sequence::GENERIC),
    ("sequence.default", sequence::DEFAULT),
];

fn definitions() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    definitions::DEFINITIONS
        .iter()
        .chain(super::conditions::DEFINITIONS.iter())
        .chain(bitwise::DEFINITIONS.iter())
        .chain(run_length::DEFINITIONS.iter())
        .chain(rowsum::DEFINITIONS.iter())
        .chain(encode_string::DEFINITIONS.iter())
        .chain(evaluation::DEFINITIONS.iter())
        .chain(environments::DEFINITIONS.iter())
        .chain(formals::DEFINITIONS.iter())
        .chain(list_values::DEFINITIONS.iter())
        .chain(printing::DEFINITIONS.iter())
        .chain(SEQUENCE_DEFINITIONS.iter())
}

fn template(name: &str, source: &str) -> Result<Sexp, EvalError> {
    let parsed = Parser
        .parse_all(source)
        .map_err(|error| EvalError::Parse(format!("base bootstrap: {}", error.message)))?;
    let [Sexp::Function { parameters, body }] = parsed.expressions.as_slice() else {
        return Err(EvalError::NotImplemented("base closure bootstrap shape"));
    };
    let mut body = body.clone();
    if let Some(kernel) = run_length::lower(name) {
        *body = kernel;
    } else if name == "encodeString" {
        *body = Sexp::InternalCall {
            operation: r_ust_sexp::InternalOperation::EncodeStringWrapper,
            arguments: Vec::new(),
        };
    } else {
        lower(&mut body)?;
    }
    Ok(Sexp::Closure {
        parameters: parameters.clone(),
        body,
        environment: ClosureEnvironment::Base,
    })
}

pub(crate) fn iter() -> Result<impl Iterator<Item = (&'static str, &'static Sexp)>, EvalError> {
    static BASE: OnceLock<Result<Vec<(&'static str, Sexp)>, EvalError>> = OnceLock::new();
    let values = BASE
        .get_or_init(|| {
            definitions()
                .map(|(name, source)| template(name, source).map(|value| (*name, value)))
                .chain(constants::iter().map(Ok))
                .chain(
                    crate::primitive_calls::names::NAMES
                        .iter()
                        .filter_map(|name| {
                            PrimitiveId::from_name(name).map(|id| Ok((*name, Sexp::Primitive(id))))
                        }),
                )
                .collect()
        })
        .as_ref()
        .map_err(Clone::clone)?;
    Ok(values.iter().map(|(name, value)| (*name, value)))
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
