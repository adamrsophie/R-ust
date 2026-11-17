use super::{aggregate, attribute, names};
use crate::{AggregateVector, AtomicVector, GraphError, ListElement, Sexp};

pub(super) fn describe(value: &Sexp) -> Result<Option<Sexp>, GraphError> {
    Ok(Some(match value {
        Sexp::Factor {
            codes,
            levels,
            ordered,
        } => {
            if codes
                .iter()
                .flatten()
                .any(|code| *code == 0 || *code > levels.len())
            {
                return Err(GraphError::WrongKind);
            }
            let values = codes
                .iter()
                .map(|code| {
                    code.map(|code| i32::try_from(code).map_err(|_| GraphError::WrongKind))
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;
            let class = if *ordered {
                vec![Some("ordered".into()), Some("factor".into())]
            } else {
                vec![Some("factor".into())]
            };
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(values),
                attributes: vec![
                    attribute(
                        "levels",
                        Sexp::String(levels.iter().map(|level| Some(level.clone())).collect()),
                    ),
                    attribute("class", Sexp::String(class)),
                ],
            }
        }
        Sexp::DataFrame { columns, nrow } => dataframe(columns, *nrow)?,
        Sexp::Matrix { data, nrow, ncol } => matrix(data, (*nrow, *ncol))?,
        _ => return Ok(None),
    }))
}

fn dataframe(columns: &[ListElement], nrow: usize) -> Result<Sexp, GraphError> {
    for column in columns {
        if super::length(&column.value)? != nrow {
            return Err(GraphError::WrongKind);
        }
    }
    let count = i32::try_from(nrow).map_err(|_| GraphError::WrongKind)?;
    let rows = attribute(
        "row.names",
        Sexp::Integer(if count == 0 {
            vec![]
        } else {
            vec![None, Some(-count)]
        }),
    );
    let class = attribute("class", Sexp::String(vec![Some("data.frame".into())]));
    let attributes = if columns.is_empty() {
        vec![names(columns), rows, class]
    } else {
        vec![names(columns), class, rows]
    };
    Ok(aggregate(
        AggregateVector::Vector(columns.iter().map(|column| column.value.clone()).collect()),
        attributes,
    ))
}

fn matrix(data: &Sexp, dimensions: (usize, usize)) -> Result<Sexp, GraphError> {
    if dimensions.0.checked_mul(dimensions.1) != Some(super::length(data)?) {
        return Err(GraphError::WrongKind);
    }
    let mut value = data.attribute_description()?.ok_or(GraphError::WrongKind)?;
    let attributes = match &mut value {
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. } => attributes,
        _ => return Err(GraphError::WrongKind),
    };
    let dimensions = Sexp::Integer(vec![
        Some(i32::try_from(dimensions.0).map_err(|_| GraphError::WrongKind)?),
        Some(i32::try_from(dimensions.1).map_err(|_| GraphError::WrongKind)?),
    ]);
    if let Some(existing) = attributes.iter().find(|a| a.name.as_deref() == Some("dim")) {
        let existing = match existing.value.snapshot()? {
            Sexp::Integer(values)
            | Sexp::AttributedAtomic {
                values: AtomicVector::Integer(values),
                ..
            } => Sexp::Integer(values),
            _ => return Err(GraphError::WrongKind),
        };
        if existing != dimensions {
            return Err(GraphError::WrongKind);
        }
    } else {
        attributes.push(attribute("dim", dimensions));
    }
    Ok(value)
}
