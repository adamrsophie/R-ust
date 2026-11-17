use super::*;
use crate::{
    AggregateVector, AtomicVector, ClosureEnvironment, FunctionParameter, ListElement, SexpType,
};
mod aggregate_attributes;
mod atomic_attributes;
mod attribute_rules;
mod restore;
mod validation;
pub(super) use validation::validate_shape;

type Source = (Sexp, Option<ValueImport>, Option<ValueKind>);

#[derive(Clone)]
pub(in crate::graph) enum Shape {
    Value(Sexp),
    Aggregate {
        expression: bool,
        length: usize,
    },
    Atomic {
        values: AtomicVector,
    },
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

pub(super) fn header(value: &Sexp) -> Result<SexpHeader, GraphError> {
    let mut header = SexpHeader::new(storage_type(value));
    header.set_object(match value {
        Sexp::AttributedAtomic { values, attributes } => {
            atomic_attributes::validate(values, attributes)?
        }
        Sexp::AttributedAggregate { values, attributes } => {
            aggregate_attributes::validate(values, attributes)?
        }
        Sexp::Factor { .. } | Sexp::DataFrame { .. } => true,
        Sexp::ClassedList { class, .. } => class_object(class)?,
        Sexp::NamedList { class, .. } => class
            .as_deref()
            .map(class_object)
            .transpose()?
            .unwrap_or(false),
        Sexp::Closure { attributes, .. } | Sexp::AttributedCharacter { attributes, .. } => {
            attributes
                .iter()
                .find(|attribute| attribute.name.as_deref() == Some("class"))
                .map(|attribute| class_object(&attribute.value))
                .transpose()?
                .unwrap_or(false)
        }
        _ => false,
    });
    Ok(header)
}

pub(super) fn kind(value: &Sexp, mode: ValueImport) -> ValueKind {
    match value {
        Sexp::Closure { .. } => ValueKind::Closure,
        Sexp::Expression { .. } => ValueKind::Expression,
        Sexp::AttributedCharacter { .. } => ValueKind::Character,
        Sexp::AttributedAtomic { .. } => ValueKind::Atomic,
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(_),
            ..
        } => ValueKind::Vector,
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(_),
            ..
        } => ValueKind::Expression,
        Sexp::Matrix { .. } => ValueKind::Matrix,
        Sexp::DataFrame { .. } => ValueKind::DataFrame,
        Sexp::Vector(_) => ValueKind::Vector,
        Sexp::List(_) | Sexp::NamedList { .. } | Sexp::ClassedList { .. } => ValueKind::List,
        Sexp::PairList(_) => ValueKind::PairList,
        Sexp::Dots(_) => ValueKind::Dots,
        Sexp::Managed(_)
        | Sexp::Nil
        | Sexp::Primitive(_)
        | Sexp::Bytecode(_)
        | Sexp::Environment(_)
        | Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Raw(_)
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
    if let Sexp::AttributedAggregate { values, attributes } = value {
        let expression = matches!(values, AggregateVector::Expression(_));
        let child_mode = expression.then_some(ValueImport::Code);
        let mut children = values
            .values()
            .iter()
            .map(|value| (value.clone(), child_mode, None))
            .collect::<Vec<_>>();
        children.push((
            Sexp::pairlist(attributes.clone()),
            Some(ValueImport::Evaluated),
            Some(ValueKind::Attributes),
        ));
        return Ok((
            Shape::Aggregate {
                expression,
                length: values.len(),
            },
            children,
        ));
    }
    if let Sexp::AttributedAtomic { values, attributes } = value {
        return Ok((
            Shape::Atomic {
                values: values.clone(),
            },
            vec![(
                Sexp::pairlist(attributes.clone()),
                Some(ValueImport::Evaluated),
                Some(ValueKind::Attributes),
            )],
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

fn class_object(value: &Sexp) -> Result<bool, GraphError> {
    match value {
        Sexp::Managed(value) => class_object(value.borrow()?.value()),
        Sexp::Nil => Ok(false),
        Sexp::String(values)
        | Sexp::AttributedCharacter { values, .. }
        | Sexp::AttributedAtomic {
            values: AtomicVector::String(values),
            ..
        } => Ok(!values.is_empty()),
        _ => Ok(true),
    }
}

pub(super) fn validate_attributes(
    kind: SexpType,
    length: usize,
    attributes: &[ListElement],
) -> Result<bool, GraphError> {
    attribute_rules::validate(kind, length, attributes)
}
