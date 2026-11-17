use super::*;

impl Shape {
    pub(in super::super) fn restore(self, values: Vec<Sexp>) -> Result<Sexp, GraphError> {
        match self {
            Self::Aggregate { expression, length } => {
                if Some(values.len()) != length.checked_add(1) {
                    return Err(GraphError::WrongKind);
                }
                let mut values = values;
                let attributes = attributes(values.pop())?;
                Ok(Sexp::AttributedAggregate {
                    values: if expression {
                        AggregateVector::Expression(values)
                    } else {
                        AggregateVector::Vector(values)
                    },
                    attributes,
                })
            }
            Self::Atomic { values: payload } => {
                let mut slots = values.into_iter();
                let attributes = attributes(slots.next())?;
                if slots.next().is_some() {
                    return Err(GraphError::WrongKind);
                }
                Ok(Sexp::AttributedAtomic {
                    values: payload,
                    attributes,
                })
            }
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
