use super::*;

pub(super) fn names_value(value: &Sexp) -> Result<Sexp, EvalError> {
    if value.sexptype() == r_ust_sexp::SexpType::Language {
        let Sexp::Call { arguments, .. } = r_ust_sexp::canonical_actual_call(value)
            .map_err(|_| EvalError::NotImplemented("names of language representation"))?
        else {
            return Err(EvalError::Type("invalid language value".into()));
        };
        return Ok(
            if arguments.iter().any(|argument| {
                argument
                    .name
                    .as_deref()
                    .is_some_and(|name| !name.is_empty())
            }) {
                Sexp::String(
                    std::iter::once(Some(String::new()))
                        .chain(
                            arguments
                                .into_iter()
                                .map(|argument| Some(argument.name.unwrap_or_default())),
                        )
                        .collect(),
                )
            } else {
                Sexp::Nil
            },
        );
    }
    if let Sexp::Expression {
        elements,
        names_present,
    } = value
    {
        return Ok(expression_values::names(elements, *names_present));
    }
    let projection = ordinary_values::read(value)?;
    if let Some(view) = projection.ordinary_list() {
        return Ok(if view.names_present {
            Sexp::String(
                view.elements
                    .iter()
                    .map(|element| element.name.clone())
                    .collect(),
            )
        } else {
            Sexp::Nil
        });
    }
    let pairlist = matches!(projection.as_ref(), Sexp::PairList(_));
    let values = match projection.as_ref() {
        Sexp::DataFrame { columns, .. } => columns.as_slice(),
        Sexp::ClassedList { elements, .. } | Sexp::List(elements) => elements.as_slice(),
        Sexp::PairList(values) => values.as_slice(),
        _ => return Ok(Sexp::Nil),
    };
    Ok(if values.iter().all(|element| element.name.is_none()) {
        Sexp::Nil
    } else {
        Sexp::String(
            values
                .iter()
                .map(|element| {
                    if pairlist {
                        Some(element.name.clone().unwrap_or_default())
                    } else {
                        element.name.clone()
                    }
                })
                .collect(),
        )
    })
}

pub(super) fn names_assign_value(value: &Sexp, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::Expression { elements, .. } = value {
        return expression_values::assign_names(elements, replacement);
    }
    if let Sexp::DataFrame { columns, nrow } = value {
        let names = string_values(replacement)?;
        if names.len() > columns.len() {
            return Err(EvalError::Domain(format!(
                "'names' attribute [{}] must be the same length as the vector [{}]",
                names.len(),
                columns.len()
            )));
        }
        let mut out = columns.clone();
        for (index, element) in out.iter_mut().enumerate() {
            element.name = names.get(index).cloned().unwrap_or(None);
        }
        return Ok(Sexp::DataFrame {
            columns: out,
            nrow: *nrow,
        });
    }
    list_attributes::assign_names(value, replacement)
}
