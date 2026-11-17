use super::*;

#[path = "corrections/dimnames.rs"]
mod dimnames;
#[path = "corrections/names.rs"]
mod names;
#[path = "corrections/replacements.rs"]
mod replacements;

fn variant(value: &Sexp, managed: bool) -> Result<Sexp> {
    Ok(if managed {
        Sexp::Managed(ValueObject::import(
            &Environment::global(),
            value,
            ValueImport::Evaluated,
        )?)
    } else {
        value.clone()
    })
}

fn get(frontend: &Frontend, name: &str) -> Result<Sexp> {
    frontend
        .runtime()
        .get(name)?
        .ok_or_else(|| "missing binding".into())
}

fn metadata(value: &Sexp) -> Result<Vec<ListElement>> {
    match value.attribute_description()? {
        Some(Sexp::AttributedAtomic { attributes, .. })
        | Some(Sexp::AttributedAggregate { attributes, .. })
        | Some(Sexp::AttributedCharacter { attributes, .. }) => Ok(attributes),
        _ => Err("missing metadata description".into()),
    }
}

fn attribute(value: &Sexp, name: &str) -> Result<Sexp> {
    Ok(metadata(value)?
        .into_iter()
        .find(|attr| attr.name.as_deref() == Some(name))
        .map(|attr| attr.value)
        .unwrap_or(Sexp::Nil))
}

fn aggregate(value: &Sexp) -> Result<(Vec<Sexp>, Vec<ListElement>)> {
    match value.attribute_description()? {
        Some(Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            attributes,
        }) => Ok((values, attributes)),
        _ => Err("missing vector description".into()),
    }
}

fn payload_shared(original: &Sexp, changed: &Sexp, count: usize) -> Result {
    if let Sexp::Managed(original) = original {
        let Sexp::Managed(changed) = changed else {
            panic!("Managed target lost its graph handle");
        };
        let before = original.view()?;
        let after = changed.view()?;
        assert!(!original.same_object(changed));
        for i in 0..count {
            assert!(before.children[i].same_object(&after.children[i]));
        }
    }
    Ok(())
}

fn slot(value: Sexp) -> ListElement {
    ListElement { name: None, value }
}

fn labels() -> Sexp {
    Sexp::String(vec![Some("a".into()), Some("b".into())])
}

fn array() -> Sexp {
    Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(2), Some(3), Some(4)]),
        attributes: vec![attr("dim", Sexp::Integer(vec![Some(2), Some(2)]))],
    }
}
