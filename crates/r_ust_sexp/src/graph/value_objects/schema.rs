use super::*;
use crate::{ClosureEnvironment, FunctionParameter, ListElement, SexpType};
mod validation;
pub(super) use validation::validate_shape;

type Source = (Sexp, Option<ValueImport>, Option<ValueKind>);

#[derive(Clone)]
pub(in crate::graph) enum Shape {
    Value(Sexp),
    Character {
        values: Vec<Option<String>>,
    },
    Closure {
        defaults: Vec<bool>,
        base_environment: bool,
    },
}

pub(super) fn storage_type(value: &Sexp) -> SexpType {
    match value {
        Sexp::Matrix { data, .. } => storage_type(data),
        _ => value.sexptype(),
    }
}

pub(super) fn kind(value: &Sexp, mode: ValueImport) -> ValueKind {
    match value {
        Sexp::Closure { .. } => ValueKind::Closure,
        Sexp::Expression { .. } => ValueKind::Expression,
        Sexp::AttributedCharacter { .. } => ValueKind::Character,
        Sexp::Matrix { .. } => ValueKind::Matrix,
        Sexp::DataFrame { .. } => ValueKind::DataFrame,
        Sexp::Vector(_) => ValueKind::Vector,
        Sexp::List(_) | Sexp::NamedList { .. } | Sexp::ClassedList { .. } => ValueKind::List,
        Sexp::PairList(_) => ValueKind::PairList,
        Sexp::Dots(_) => ValueKind::Dots,
        Sexp::Nil
        | Sexp::Primitive(_)
        | Sexp::Bytecode(_)
        | Sexp::Environment(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::Char(_)
        | Sexp::Factor { .. }
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Promise { .. } => ValueKind::Leaf,
        _ => {
            let _ = mode;
            ValueKind::Code
        }
    }
}

pub(super) fn split(value: &Sexp) -> Result<(Shape, Vec<Source>), GraphError> {
    if let Sexp::Closure {
        parameters,
        body,
        environment,
        attributes,
    } = value
    {
        let formals = Sexp::pairlist(
            parameters
                .iter()
                .map(|p| ListElement {
                    name: Some(p.name.clone()),
                    value: p.default.clone().unwrap_or(Sexp::MissingValue),
                })
                .collect(),
        );
        let captured = match environment {
            ClosureEnvironment::Base => Sexp::Nil,
            ClosureEnvironment::Captured(value) => Sexp::Environment(value.clone()),
        };
        return Ok((
            Shape::Closure {
                defaults: parameters.iter().map(|p| p.default.is_some()).collect(),
                base_environment: matches!(environment, ClosureEnvironment::Base),
            },
            vec![
                (formals, Some(ValueImport::Code), Some(ValueKind::Formals)),
                (*body.clone(), Some(ValueImport::Code), None),
                (captured, Some(ValueImport::Evaluated), None),
                (
                    Sexp::pairlist(attributes.clone()),
                    Some(ValueImport::Evaluated),
                    Some(ValueKind::Attributes),
                ),
            ],
        ));
    }
    if let Sexp::AttributedCharacter { values, attributes } = value {
        return Ok((
            Shape::Character {
                values: values.clone(),
            },
            vec![(
                Sexp::pairlist(attributes.clone()),
                Some(ValueImport::Evaluated),
                Some(ValueKind::Attributes),
            )],
        ));
    }
    let mut shape = value.clone();
    let mut sources = Vec::new();
    let child_mode = match value {
        Sexp::Expression { .. } => Some(ValueImport::Code),
        Sexp::AttributedCharacter { .. } => Some(ValueImport::Evaluated),
        _ => None,
    };
    children::children(&mut shape, &mut |child| {
        sources.push((std::mem::replace(child, Sexp::Nil), child_mode, None));
        Ok(())
    })?;
    Ok((Shape::Value(shape), sources))
}

impl Shape {
    pub(super) fn restore(self, values: Vec<Sexp>) -> Result<Sexp, GraphError> {
        match self {
            Self::Character { values: text } => {
                let mut slots = values.into_iter();
                let attributes = attributes(slots.next())?;
                if slots.next().is_some() {
                    return Err(GraphError::WrongKind);
                }
                Ok(Sexp::AttributedCharacter {
                    values: text,
                    attributes,
                })
            }
            Self::Value(mut value) => {
                let mut values = values.into_iter();
                children::children(&mut value, &mut |child| {
                    *child = values.next().ok_or(GraphError::WrongKind)?;
                    Ok(())
                })?;
                if values.next().is_some() {
                    return Err(GraphError::WrongKind);
                }
                Ok(value)
            }
            Self::Closure {
                defaults,
                base_environment,
            } => {
                let mut slots = values.into_iter();
                let formals = match slots.next() {
                    Some(Sexp::Nil) => Vec::new(),
                    Some(Sexp::PairList(value)) => value.as_slice().to_vec(),
                    _ => return Err(GraphError::WrongKind),
                };
                if formals.len() != defaults.len() {
                    return Err(GraphError::WrongKind);
                }
                let parameters = formals
                    .into_iter()
                    .zip(defaults)
                    .map(|(p, has_default)| {
                        Ok(FunctionParameter {
                            name: p.name.ok_or(GraphError::WrongKind)?,
                            default: if has_default { Some(p.value) } else { None },
                        })
                    })
                    .collect::<Result<Vec<_>, GraphError>>()?;
                let body = Box::new(slots.next().ok_or(GraphError::WrongKind)?);
                let environment = match (base_environment, slots.next()) {
                    (true, Some(Sexp::Nil)) => ClosureEnvironment::Base,
                    (false, Some(Sexp::Environment(value))) => ClosureEnvironment::Captured(value),
                    _ => return Err(GraphError::WrongKind),
                };
                let attributes = attributes(slots.next())?;
                if slots.next().is_some() {
                    return Err(GraphError::WrongKind);
                }
                Ok(Sexp::Closure {
                    parameters,
                    body,
                    environment,
                    attributes,
                })
            }
        }
    }
}

fn attributes(value: Option<Sexp>) -> Result<Vec<ListElement>, GraphError> {
    match value {
        Some(Sexp::Nil) => Ok(Vec::new()),
        Some(Sexp::PairList(value)) => Ok(value.as_slice().to_vec()),
        _ => Err(GraphError::WrongKind),
    }
}
