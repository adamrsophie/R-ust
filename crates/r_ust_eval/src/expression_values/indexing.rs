use crate::{EvalError, ListElement, Sexp};

type Expression<'a> = (&'a [ListElement], bool);

pub(crate) fn select(expression: Expression<'_>, positions: &[Option<usize>]) -> Sexp {
    let (elements, names_present) = expression;
    let elements = positions
        .iter()
        .map(|position| {
            position
                .and_then(|position| elements.get(position))
                .cloned()
                .unwrap_or(ListElement {
                    name: None,
                    value: Sexp::Nil,
                })
        })
        .collect();
    Sexp::Expression {
        elements,
        names_present,
    }
}

pub(crate) fn subset(expression: Expression<'_>, index: Option<&Sexp>) -> Result<Sexp, EvalError> {
    let (elements, names_present) = expression;
    let Some(index) = index else {
        return Ok(Sexp::Expression {
            elements: elements.to_vec(),
            names_present,
        });
    };
    let positions = if let Some(names) = index.character_values() {
        names
            .iter()
            .map(|name| {
                name.as_deref()
                    .filter(|name| names_present && !name.is_empty())
                    .and_then(|name| {
                        elements
                            .iter()
                            .position(|element| element.name.as_deref() == Some(name))
                    })
            })
            .collect()
    } else {
        crate::subset_positions(elements.len(), index)?
    };
    Ok(select(expression, &positions))
}

pub(crate) fn extract(expression: Expression<'_>, index: &Sexp) -> Result<Sexp, EvalError> {
    let (elements, names_present) = expression;
    if let Some(names) = index.character_values() {
        let [name] = names else {
            return Err(EvalError::NotImplemented(
                "recursive expression extraction by names",
            ));
        };
        let value = name
            .as_deref()
            .filter(|name| names_present && !name.is_empty())
            .and_then(|name| {
                elements
                    .iter()
                    .find(|element| element.name.as_deref() == Some(name))
            })
            .map(|element| element.value.clone())
            .unwrap_or(Sexp::Nil);
        return Ok(value);
    }
    if crate::value_len(index)? != 1 {
        return Err(EvalError::NotImplemented(
            "recursive expression extraction by positions",
        ));
    }
    let position = crate::extraction_position(index)?;
    elements
        .get(position)
        .map(|element| element.value.clone())
        .ok_or_else(|| EvalError::Domain("subscript out of bounds".into()))
}
