use crate::{EvalError, EvaluatedArgument, ListElement, Sexp};

pub(super) fn call_list(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let elements = args
        .iter()
        .map(|arg| ListElement {
            name: arg.name.clone(),
            value: arg.value.clone(),
        })
        .collect::<Vec<_>>();
    Ok(from_tags(&elements))
}

pub(super) fn from_tags(tags: &[ListElement]) -> Sexp {
    let present = tags
        .iter()
        .any(|element| element.name.as_deref().is_some_and(|name| !name.is_empty()));
    let mut elements = tags.to_vec();
    if present {
        for element in &mut elements {
            if element.name.is_none() {
                element.name = Some(String::new());
            }
        }
    }
    Sexp::list_with_metadata(elements, present, None)
}

pub(super) fn to_pairlist(view: r_ust_sexp::OrdinaryListView<'_>) -> Sexp {
    if view.elements.is_empty() {
        return Sexp::Nil;
    }
    Sexp::pairlist(
        view.elements
            .iter()
            .map(|element| ListElement {
                name: if view.names_present {
                    Some(element.name.clone().unwrap_or_else(|| "NA".into()))
                        .filter(|name| !name.is_empty())
                } else {
                    None
                },
                value: element.value.clone(),
            })
            .collect(),
    )
}
