use super::*;
use crate::{EvaluatedArgument, RuntimeState};

impl RuntimeState {
    pub(crate) fn set_runtime_attribute(
        &mut self,
        object: &Sexp,
        update: (&str, &Sexp),
    ) -> Result<Sexp, EvalError> {
        let (name, value) = update;
        if description(object)?.is_none() {
            return crate::list_attributes::set(object, name, value);
        }
        if matches!(value, Sexp::Nil) {
            return set_prepared(object, name, value);
        }
        let plain = payload(value)?;
        let call = self.source_current_call();
        let prepared = match name {
            "dim"
                if matches!(
                    plain,
                    Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_)
                ) =>
            {
                let integers = Sexp::Integer(self.integer_values_with_warning(&plain, &call)?);
                normalize::value(object, (name, &integers))?
            }
            "row.names" if matches!(&plain,Sexp::Real(values) if values.len()==2 && values[0].is_nan()) =>
            {
                let integers = Sexp::Integer(self.integer_values_with_warning(&plain, &call)?);
                rows::normalize(&integers)?
            }
            "tsp" => {
                let tolerance = self
                    .get_option("ts.eps")
                    .map(|value| crate::numeric_scalar(value, "ts.eps"))
                    .transpose()?
                    .unwrap_or(f64::NAN);
                normalize::tsp(object, value, tolerance)?
            }
            _ => normalize::value(object, update)?,
        };
        set_prepared(object, name, &prepared)
    }

    pub(crate) fn replace_runtime_attributes(
        &mut self,
        object: &Sexp,
        replacements: &Sexp,
    ) -> Result<Sexp, EvalError> {
        if description(object)?.is_none() && !matches!(object, Sexp::Nil) {
            return crate::list_attributes::replace(object, replacements);
        }
        let attrs = attribute_list(replacements)?;
        let object = if matches!(object, Sexp::Nil) && !matches!(replacements, Sexp::Nil) {
            Sexp::List(vec![])
        } else {
            object.clone()
        };
        let mut result = clear(&object)?;
        for attr in ordered(&attrs) {
            result = self.set_runtime_attribute(
                &result,
                (attr.name.as_deref().unwrap_or("NA"), &attr.value),
            )?;
        }
        Ok(result)
    }

    pub(crate) fn assign_runtime_names(
        &mut self,
        object: &Sexp,
        replacement: &Sexp,
    ) -> Result<Sexp, EvalError> {
        if matches!(replacement, Sexp::Nil) && description(object)?.is_some() {
            let attrs = stored(object)?;
            return set_prepared(
                object,
                if dimensions::one_dimensional(&attrs)? {
                    "dimnames"
                } else {
                    "names"
                },
                replacement,
            );
        }
        let plain_character = matches!(replacement, Sexp::String(_));
        let value = if matches!(replacement, Sexp::Nil) || plain_character {
            replacement.clone()
        } else {
            let attrs = stored(replacement)?;
            let factor = find(&attrs, "class")
                .map(payload)
                .transpose()?
                .as_ref()
                .and_then(Sexp::character_values)
                .is_some_and(|classes| {
                    classes
                        .iter()
                        .any(|class| class.as_deref() == Some("factor"))
                });
            if factor {
                dimensions::simple_character(replacement)?
            } else {
                let call = self.source_current_call();
                self.call_character_coercion(
                    &[EvaluatedArgument {
                        name: None,
                        value: replacement.clone(),
                    }],
                    &call,
                )?
            }
        };
        crate::names_assign_value(object, &value)
    }
}
