mod compatibility;
mod rows;
pub use rows::expanded_row_names;

use crate::{AggregateVector, AtomicVector, GraphError, ListElement, Sexp};

impl Sexp {
    /// Explicit SDK normalization. Stored attributes stay ordered and row names stay compact.
    pub fn attribute_description(&self) -> Result<Option<Sexp>, GraphError> {
        match self {
            Self::Managed(value) => value.borrow()?.value().attribute_description(),
            Self::ListObject(value) => value.value()?.attribute_description(),
            Self::AttributedAtomic { .. }
            | Self::AttributedAggregate { .. }
            | Self::AttributedCharacter { .. }
            | Self::Closure { .. } => Ok(Some(self.clone())),
            Self::Logical(values) => Ok(Some(atomic(AtomicVector::Logical(values.clone())))),
            Self::Integer(values) => Ok(Some(atomic(AtomicVector::Integer(values.clone())))),
            Self::Real(values) => Ok(Some(atomic(AtomicVector::Real(values.clone())))),
            Self::Raw(values) => Ok(Some(atomic(AtomicVector::Raw(values.clone())))),
            Self::String(values) => Ok(Some(atomic(AtomicVector::String(values.clone())))),
            Self::Vector(values) => Ok(Some(aggregate(
                AggregateVector::Vector(values.clone()),
                vec![],
            ))),
            Self::Expression {
                elements,
                names_present,
            } => {
                let mut attributes = Vec::new();
                if *names_present {
                    attributes.push(names(elements));
                }
                Ok(Some(aggregate(
                    AggregateVector::Expression(elements.iter().map(|e| e.value.clone()).collect()),
                    attributes,
                )))
            }
            Self::Factor { .. } | Self::DataFrame { .. } | Self::Matrix { .. } => {
                compatibility::describe(self)
            }
            _ => Ok(self.ordinary_list().map(|view| {
                let mut attributes = Vec::new();
                if view.class_first {
                    if let Some(class) = view.class {
                        attributes.push(attribute("class", class.clone()));
                    }
                }
                if view.names_present {
                    attributes.push(names(view.elements));
                }
                if !view.class_first {
                    if let Some(class) = view.class {
                        attributes.push(attribute("class", class.clone()));
                    }
                }
                aggregate(
                    AggregateVector::Vector(
                        view.elements.iter().map(|e| e.value.clone()).collect(),
                    ),
                    attributes,
                )
            })),
        }
    }
}

fn atomic(values: AtomicVector) -> Sexp {
    Sexp::AttributedAtomic {
        values,
        attributes: vec![],
    }
}

fn aggregate(values: AggregateVector, attributes: Vec<ListElement>) -> Sexp {
    Sexp::AttributedAggregate { values, attributes }
}

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

fn names(elements: &[ListElement]) -> ListElement {
    attribute(
        "names",
        Sexp::String(elements.iter().map(|e| e.name.clone()).collect()),
    )
}

pub(crate) fn length(value: &Sexp) -> Result<usize, GraphError> {
    match value {
        Sexp::Managed(value) => length(value.borrow()?.value()),
        Sexp::Nil => Ok(0),
        Sexp::ListObject(value) => length(&value.value()?),
        Sexp::AttributedAtomic { values, .. } => Ok(values.len()),
        Sexp::AttributedAggregate { values, .. } => Ok(values.len()),
        Sexp::Logical(values) => Ok(values.len()),
        Sexp::Integer(values) => Ok(values.len()),
        Sexp::Real(values) => Ok(values.len()),
        Sexp::Raw(values) => Ok(values.len()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => Ok(values.len()),
        Sexp::Vector(values) => Ok(values.len()),
        Sexp::List(values)
        | Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::Expression {
            elements: values, ..
        } => Ok(values.len()),
        Sexp::Factor { codes, .. } => Ok(codes.len()),
        Sexp::Matrix { data, .. } => length(data),
        _ => Err(GraphError::WrongKind),
    }
}
