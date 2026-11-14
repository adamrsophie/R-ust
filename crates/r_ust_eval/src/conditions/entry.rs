use super::{bad_stack, ListElement, ListObject};
use crate::{EvalError, Sexp};

pub(super) struct Entry {
    pub(super) object: ListObject,
    pub(super) class: Option<String>,
    pub(super) handler: Sexp,
    pub(super) target: Sexp,
    pub(super) result: ListObject,
}

impl Entry {
    pub(super) fn read(value: Sexp) -> Result<Self, EvalError> {
        let Sexp::ListObject(object) = value else {
            return Err(bad_stack());
        };
        let values = list_values(&object)?;
        let [class, parent, handler, target, result] = values.as_slice() else {
            return Err(bad_stack());
        };
        let Sexp::Char(class) = &class.value else {
            return Err(bad_stack());
        };
        if !matches!(parent.value, Sexp::Nil | Sexp::Environment(_))
            || !matches!(target.value, Sexp::Nil | Sexp::Environment(_))
        {
            return Err(bad_stack());
        }
        let Sexp::ListObject(result) = &result.value else {
            return Err(bad_stack());
        };
        let result_values = list_values(result)?;
        let [_, _, _, token] = result_values.as_slice() else {
            return Err(bad_stack());
        };
        let Sexp::ListObject(token) = &token.value else {
            return Err(bad_stack());
        };
        let token_values = list_values(token)?;
        if !matches!(token_values.as_slice(), [value] if matches!(value.value, Sexp::Nil)) {
            return Err(bad_stack());
        }
        Ok(Self {
            object,
            class: class.clone(),
            handler: handler.value.clone(),
            target: target.value.clone(),
            result: result.clone(),
        })
    }

    pub(super) fn calling(&self) -> Result<bool, EvalError> {
        Ok(self.object.header()?.gp() != 0)
    }
}

fn list_values(object: &ListObject) -> Result<Vec<ListElement>, EvalError> {
    let (Sexp::List(values)
    | Sexp::ClassedList {
        elements: values, ..
    }) = object.value()?
    else {
        return Err(bad_stack());
    };
    Ok(values)
}
