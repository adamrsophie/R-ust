use crate::{EvalError, ListElement, RuntimeState, Sexp};
use r_ust_sexp::ListObject;

impl RuntimeState {
    pub(crate) fn intern_list(&self, value: Sexp) -> Result<Sexp, EvalError> {
        let Some(view) = value.ordinary_list() else {
            return Ok(value);
        };
        let elements = view
            .elements
            .iter()
            .map(|slot| {
                Ok(ListElement {
                    name: slot.name.clone(),
                    value: self.intern_list(slot.value.clone())?,
                })
            })
            .collect::<Result<Vec<_>, EvalError>>()?;
        let mut value = value;
        match &mut value {
            Sexp::List(slots)
            | Sexp::NamedList {
                elements: slots, ..
            }
            | Sexp::ClassedList {
                elements: slots, ..
            } => *slots = elements,
            _ => unreachable!("ordinary list projection"),
        }
        Ok(Sexp::ListObject(ListObject::from_runtime_value(
            &self.current,
            &value,
        )?))
    }
}

pub(crate) fn expose(value: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::ListObject(object) = value {
        return if object.is_runtime_value()? {
            expose(&object.value()?)
        } else {
            Ok(value.clone())
        };
    }
    let mut value = value.clone();
    match &mut value {
        Sexp::List(slots)
        | Sexp::NamedList {
            elements: slots, ..
        }
        | Sexp::ClassedList {
            elements: slots, ..
        }
        | Sexp::Expression {
            elements: slots, ..
        }
        | Sexp::Dots(slots)
        | Sexp::DataFrame { columns: slots, .. } => expose_slots(slots)?,
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => expose_slots(attributes)?,
        Sexp::AttributedAggregate { values, attributes } => {
            for value in values.values_mut() {
                *value = expose(value)?;
            }
            expose_slots(attributes)?;
        }
        Sexp::Vector(values) => {
            for value in values {
                *value = expose(value)?;
            }
        }
        Sexp::Matrix { data, .. } => **data = expose(data)?,
        Sexp::PairList(slots) => {
            let mut exposed = slots.as_slice().to_vec();
            expose_slots(&mut exposed)?;
            value = Sexp::pairlist(exposed);
        }
        Sexp::Call {
            function,
            arguments,
        } => {
            **function = expose(function)?;
            for arg in arguments {
                arg.value = expose(&arg.value)?;
            }
        }
        _ => {}
    }
    Ok(value)
}

fn expose_slots(slots: &mut [ListElement]) -> Result<(), EvalError> {
    for slot in slots {
        slot.value = expose(&slot.value)?;
    }
    Ok(())
}
