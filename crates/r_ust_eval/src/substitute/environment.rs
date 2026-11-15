use crate::{EvalError, ListElement, Sexp, SourceFault};
use r_ust_sexp::{Environment, EnvironmentRole};

pub(super) enum Lookup {
    Disabled,
    Frame(Environment),
    Entries(Vec<ListElement>),
}

impl Lookup {
    pub(super) fn new(value: &Sexp, call: &Sexp) -> Result<Self, EvalError> {
        let projection = crate::ordinary_values::read(value)?;
        if let Some(view) = projection.ordinary_list() {
            return Ok(Self::Entries(list_names(view.elements, view.names_present)));
        }
        match projection.as_ref() {
            Sexp::Nil => Ok(Self::Disabled),
            Sexp::Environment(frame) => {
                if frame.role()? == EnvironmentRole::Global {
                    Ok(Self::Disabled)
                } else {
                    Ok(Self::Frame(frame.clone()))
                }
            }
            Sexp::PairList(values) => Ok(Self::Entries(values.as_slice().to_vec())),
            Sexp::List(values)
            | Sexp::ClassedList {
                elements: values, ..
            }
            | Sexp::DataFrame {
                columns: values, ..
            } => Ok(Self::Entries(list_names(
                values,
                values.iter().any(|element| element.name.is_some()),
            ))),
            Sexp::Vector(values) => Ok(Self::Entries(
                values
                    .iter()
                    .map(|value| ListElement {
                        name: None,
                        value: value.clone(),
                    })
                    .collect(),
            )),
            Sexp::Matrix { data, .. } => Self::new(data, call),
            _ => Err(SourceFault::type_error(
                "invalid environment specified".into(),
                call,
            )),
        }
    }

    pub(super) fn get(&self, name: &str) -> Result<Option<Sexp>, EvalError> {
        match self {
            Self::Disabled => Ok(None),
            Self::Frame(frame) => Ok(frame.get_local(name)?),
            Self::Entries(values) => Ok(values
                .iter()
                .find(|element| element.name.as_deref() == Some(name))
                .map(|element| element.value.clone())),
        }
    }
}

fn list_names(values: &[ListElement], named: bool) -> Vec<ListElement> {
    values
        .iter()
        .map(|element| ListElement {
            name: if named {
                match element.name.as_deref() {
                    Some("") => None,
                    Some(name) => Some(name.into()),
                    None => Some("NA".into()),
                }
            } else {
                None
            },
            value: element.value.clone(),
        })
        .collect()
}
