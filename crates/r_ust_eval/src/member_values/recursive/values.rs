use super::*;
use crate::list_attributes::engine;

pub(super) fn slots(value: &Sexp) -> Result<Option<Vec<ListElement>>, EvalError> {
    let value = crate::ordinary_values::read(value)?;
    Ok(match value.as_ref() {
        Sexp::Nil => Some(Vec::new()),
        Sexp::List(v)
        | Sexp::NamedList { elements: v, .. }
        | Sexp::ClassedList { elements: v, .. }
        | Sexp::Dots(v)
        | Sexp::DataFrame { columns: v, .. } => Some(v.clone()),
        Sexp::Expression { elements, .. } => Some(elements.clone()),
        Sexp::PairList(v) => Some(v.as_slice().to_vec()),
        Sexp::Vector(v) => Some(
            v.iter()
                .cloned()
                .map(|value| ListElement { name: None, value })
                .collect(),
        ),
        Sexp::AttributedAggregate { values, .. } => {
            let names = engine::payload(&crate::names_value(value.as_ref())?)?;
            let names = names.character_values();
            Some(
                values
                    .values()
                    .iter()
                    .enumerate()
                    .map(|(i, v)| ListElement {
                        name: names.and_then(|n| n.get(i)).cloned().flatten(),
                        value: v.clone(),
                    })
                    .collect(),
            )
        }
        value if value.sexptype() == r_ust_sexp::SexpType::Language => {
            Some(super::super::language::elements(value)?)
        }
        _ => None,
    })
}

pub(super) fn scalars(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    if let Some(slots) = slots(value)? {
        return Ok(slots);
    }
    let storage = crate::coercion::storage::read(value)?;
    let values = match storage.as_ref() {
        Sexp::Logical(v) => v.iter().map(|v| Sexp::Logical(vec![*v])).collect(),
        Sexp::Integer(v) => v.iter().map(|v| Sexp::Integer(vec![*v])).collect(),
        Sexp::Real(v) => v.iter().map(|v| Sexp::Real(vec![*v])).collect(),
        Sexp::Raw(v) => v.iter().map(|v| Sexp::Raw(vec![*v])).collect(),
        Sexp::String(v) | Sexp::AttributedCharacter { values: v, .. } => {
            v.iter().map(|v| Sexp::String(vec![v.clone()])).collect()
        }
        Sexp::Factor { codes, .. } => codes
            .iter()
            .map(|code| {
                Ok(Sexp::Integer(vec![code
                    .map(i32::try_from)
                    .transpose()
                    .map_err(|_| {
                        EvalError::Domain("invalid factor code".into())
                    })?]))
            })
            .collect::<Result<_, EvalError>>()?,
        // Recursive [[ uses the environment's internal vector length, not binding count.
        Sexp::Environment(_) => Vec::new(),
        _ => vec![Sexp::Nil; crate::value_len(storage.as_ref())?],
    };
    let names = engine::payload(&crate::names_value(value)?)?;
    let names = names.character_values();
    Ok(values
        .into_iter()
        .enumerate()
        .map(|(i, value)| ListElement {
            name: names.and_then(|v| v.get(i)).cloned().flatten(),
            value,
        })
        .collect())
}

pub(super) fn restore(original: &Sexp, slots: Vec<ListElement>) -> Result<Sexp, EvalError> {
    let projection = crate::ordinary_values::read(original)?;
    let mut value = projection.into_owned();
    match &mut value {
        Sexp::Nil | Sexp::PairList(_) => value = Sexp::pairlist(slots),
        Sexp::List(v)
        | Sexp::NamedList { elements: v, .. }
        | Sexp::ClassedList { elements: v, .. }
        | Sexp::Dots(v)
        | Sexp::DataFrame { columns: v, .. } => *v = slots,
        Sexp::Expression {
            elements,
            names_present,
        } => {
            *names_present |= slots.iter().any(|slot| slot.name.is_some());
            *elements = slots;
        }
        Sexp::Vector(v) => {
            let names = slots
                .iter()
                .any(|slot| slot.name.is_some())
                .then(|| Sexp::String(slots.iter().map(|slot| slot.name.clone()).collect()));
            *v = slots.into_iter().map(|slot| slot.value).collect();
            if let Some(names) = names {
                value = Sexp::AttributedAggregate {
                    values: r_ust_sexp::AggregateVector::Vector(v.clone()),
                    attributes: vec![ListElement {
                        name: Some("names".into()),
                        value: names,
                    }],
                };
            }
        }
        Sexp::AttributedAggregate { values, attributes } => {
            let old_len = values.len();
            let named = slots.iter().any(|slot| slot.name.is_some())
                || attributes
                    .iter()
                    .any(|a| a.name.as_deref() == Some("names"));
            let names = Sexp::String(slots.iter().map(|slot| slot.name.clone()).collect());
            let contents = slots.into_iter().map(|slot| slot.value).collect();
            match values {
                r_ust_sexp::AggregateVector::Vector(v)
                | r_ust_sexp::AggregateVector::Expression(v) => *v = contents,
            }
            if values.len() != old_len {
                attributes.retain(|a| !matches!(a.name.as_deref(), Some("dim" | "dimnames")));
            }
            let old_names = engine::payload(&crate::names_value(original)?)?;
            if named && names.character_values() != old_names.character_values() {
                if let Some(attr) = attributes
                    .iter_mut()
                    .find(|a| a.name.as_deref() == Some("names"))
                {
                    attr.value = names;
                } else {
                    attributes.push(ListElement {
                        name: Some("names".into()),
                        value: names,
                    });
                }
            }
        }
        value if value.sexptype() == r_ust_sexp::SexpType::Language => {
            return super::super::language::restore(slots)
        }
        _ => {
            return Err(EvalError::Type(
                "invalid recursive replacement container".into(),
            ))
        }
    }
    crate::ordinary_values::replaced(original, value)
}
