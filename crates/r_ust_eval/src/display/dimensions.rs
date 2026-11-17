use crate::{EvalError, ListElement, Sexp};

pub(super) struct Dimensions {
    pub sizes: Vec<usize>,
    pub labels: Vec<Option<Vec<Option<String>>>>,
    pub titles: Vec<Option<String>>,
}

pub(super) fn read(attributes: &[ListElement]) -> Result<Option<Dimensions>, EvalError> {
    let Some(dim) = super::metadata::get(attributes, "dim")? else {
        return Ok(None);
    };
    let sizes = match dim {
        Sexp::Integer(values)
        | Sexp::AttributedAtomic {
            values: r_ust_sexp::AtomicVector::Integer(values),
            ..
        } => values
            .iter()
            .map(|v| {
                v.and_then(|v| usize::try_from(v).ok())
                    .ok_or_else(|| EvalError::Type("invalid print dimensions".into()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(EvalError::Type("invalid print dimensions".into())),
    };
    if sizes.is_empty() {
        return Err(EvalError::Type("empty print dimensions".into()));
    }
    let mut labels = vec![None; sizes.len()];
    let mut titles = vec![None; sizes.len()];
    if let Some(names) = super::metadata::get(attributes, "dimnames")? {
        let (values, names) = match names {
            Sexp::AttributedAggregate {
                values: r_ust_sexp::AggregateVector::Vector(values),
                attributes,
            } => {
                let names = super::metadata::get(&attributes, "names")?
                    .map(|v| super::metadata::strings(&v))
                    .transpose()?;
                (values, names)
            }
            Sexp::Vector(values) => (values, None),
            other => {
                let view = other
                    .ordinary_list()
                    .ok_or_else(|| EvalError::Type("invalid print dimnames".into()))?;
                (
                    view.elements.iter().map(|v| v.value.clone()).collect(),
                    view.names_present
                        .then(|| view.elements.iter().map(|v| v.name.clone()).collect()),
                )
            }
        };
        if values.len() != sizes.len() {
            return Err(EvalError::Type("invalid print dimnames length".into()));
        }
        for (index, value) in values.iter().enumerate() {
            let value = super::projection::read(value)?;
            if !matches!(value.as_ref(), Sexp::Nil) {
                let strings = super::metadata::strings(value.as_ref())?;
                if strings.len() != sizes[index] {
                    return Err(EvalError::Type("invalid print dimension labels".into()));
                }
                labels[index] = Some(strings);
            }
        }
        if let Some(names) = names {
            titles = names;
        }
    }
    Ok(Some(Dimensions {
        sizes,
        labels,
        titles,
    }))
}
