use super::Formatting;
use crate::{CallArgument, EvalError, EvalFlow, Sexp};

impl Formatting<'_> {
    pub(super) fn star(&mut self, index: usize, row: usize) -> Result<i32, EvalError> {
        self.used[index] = true;
        if row == 0 && matches!(self.arguments[index], Sexp::Real(_)) {
            self.arguments[index] = Sexp::Integer(
                self.runtime
                    .integer_values_with_warning(&self.arguments[index], &self.call)?,
            );
        }
        let Sexp::Integer(values) = &self.arguments[index] else {
            return Err(
                self.error("argument for '*' conversion specification must be a number".into())
            );
        };
        values
            .get(row % values.len())
            .copied()
            .flatten()
            .ok_or_else(|| {
                self.error("argument for '*' conversion specification must be a number".into())
            })
    }

    pub(super) fn coerce(&mut self, index: usize, conversion: char) -> Result<(), EvalError> {
        match conversion {
            'd' | 'i' | 'o' | 'x' | 'X' => {
                if let Sexp::Real(values) = &self.arguments[index] {
                    if values.iter().all(|value| {
                        r_ust_sexp::is_na_real(*value)
                            || crate::real_to_integer(*value)
                                .is_some_and(|integer| f64::from(integer) == *value)
                    }) {
                        self.arguments[index] = Sexp::Integer(
                            values
                                .iter()
                                .map(|value| crate::real_to_integer(*value))
                                .collect(),
                        );
                    }
                }
            }
            'a' | 'A' | 'e' | 'f' | 'g' | 'E' | 'G' => {
                if !matches!(self.arguments[index], Sexp::Real(_))
                    && self.arguments[index].character_values().is_none()
                {
                    self.call_coercion(index, "as.double")?;
                }
            }
            's' => {
                if self.arguments[index].character_values().is_none() {
                    self.call_coercion(index, "as.character")?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn call_coercion(&mut self, index: usize, name: &str) -> Result<(), EvalError> {
        let expression = Sexp::Call {
            function: Box::new(Sexp::Symbol(name.into())),
            arguments: vec![CallArgument {
                name: None,
                value: self.arguments[index].clone(),
            }],
        };
        let EvalFlow::Value(value) = self.runtime.eval_flow(&expression)?;
        let value = crate::ordinary_values::read(&value)?.into_owned();
        let length = crate::value_len(&value)?;
        if length == 0 {
            return Err(self.error("coercion has changed vector length to 0".into()));
        }
        let check = self.lengths[index] == self.maximum;
        self.lengths[index] = length;
        self.arguments[index] = value;
        if check && length < self.maximum {
            self.recycling()?;
        }
        Ok(())
    }
}
