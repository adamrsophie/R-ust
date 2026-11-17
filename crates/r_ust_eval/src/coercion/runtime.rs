use super::{atomic, lists, strings::BlankPolicy};
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn integer_values_with_warning(
        &mut self,
        value: &Sexp,
        call: &Sexp,
    ) -> Result<Vec<Option<i32>>, EvalError> {
        let (values, warnings) = atomic::integers(value, BlankPolicy::environment())?;
        self.record_warnings_at(&warnings.messages(), Some(call))?;
        Ok(values)
    }

    pub(crate) fn raw_values_with_warning(
        &mut self,
        value: &Sexp,
        call: &Sexp,
    ) -> Result<Vec<u8>, EvalError> {
        let converted = lists::convert(value, (true, BlankPolicy::environment()), |warnings| {
            self.record_warnings_at(&warnings.messages(), Some(call))
        })?;
        let Sexp::Integer(values) = converted else {
            return Err(EvalError::Type(
                "integer coercion returned non-integer storage".into(),
            ));
        };
        let mut out_of_range = false;
        let values = values
            .into_iter()
            .map(|value| match value.and_then(|v| u8::try_from(v).ok()) {
                Some(value) => value,
                None => {
                    out_of_range = true;
                    0
                }
            })
            .collect();
        if out_of_range {
            self.record_warnings_at(
                &["out-of-range values treated as 0 in coercion to raw".into()],
                Some(call),
            )?;
        }
        Ok(values)
    }

    pub(crate) fn scalar_real_with_warning(&mut self, value: &Sexp) -> Result<f64, EvalError> {
        let projection = super::storage::read(value)?;
        let (number, warnings) =
            atomic::scalar_real(projection.as_ref(), BlankPolicy::environment());
        self.record_warnings_at(&warnings.messages(), Some(&self.source_current_call()))?;
        Ok(number)
    }

    pub(crate) fn scalar_integer_with_warning(
        &mut self,
        value: &Sexp,
    ) -> Result<Option<i32>, EvalError> {
        let projection = super::storage::read(value)?;
        let (integer, warnings) =
            atomic::scalar_integer(projection.as_ref(), BlankPolicy::environment());
        self.record_warnings_at(&warnings.messages(), Some(&self.source_current_call()))?;
        Ok(integer)
    }

    pub(crate) fn call_numeric_coercion(
        &mut self,
        name: &str,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let integer = name == "as.integer";
        let Some(first) = args.first() else {
            return Ok(if integer {
                Sexp::Integer(Vec::new())
            } else {
                Sexp::Real(Vec::new())
            });
        };
        if let Some(name) = first.name.as_deref().filter(|name| *name != "x") {
            return Err(EvalError::Arity(format!(
                "supplied argument name '{name}' does not match 'x'"
            )));
        }
        if matches!(first.value, Sexp::MissingArg(_)) {
            return Err(EvalError::Arity("argument 1 is empty".into()));
        }
        lists::convert(
            &first.value,
            (integer, BlankPolicy::environment()),
            |warnings| self.record_parse_warnings(&warnings.messages()),
        )
    }
}
