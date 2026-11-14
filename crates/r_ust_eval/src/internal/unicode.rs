use super::logical::as_logical;
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn utf8_to_int(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        if let Sexp::Matrix { data, .. } = value {
            return self.utf8_to_int(data);
        }
        let (Sexp::String(values) | Sexp::AttributedCharacter { values, .. }) = value else {
            return Err(EvalError::Domain(
                "argument must be a character vector of length 1".into(),
            ));
        };
        let Some(value) = values.first() else {
            return Err(EvalError::Domain(
                "argument must be a character vector of length 1".into(),
            ));
        };
        if values.len() > 1 {
            self.record_parse_warnings(&["argument should be a character vector of length 1\nall but the first element will be ignored".into()])?;
        }
        let Some(value) = value else {
            return Ok(Sexp::Integer(vec![None]));
        };
        // shortcut: strings store valid UTF-8 only, add byte validation with encoded-string storage.
        let integers = value
            .chars()
            .take_while(|character| *character != '\0')
            .map(|character| {
                i32::try_from(u32::from(character))
                    .map(Some)
                    .map_err(|_| EvalError::Domain("invalid UTF-8 string".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Sexp::Integer(integers))
    }

    pub(super) fn int_to_utf8(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        if matches!(args[0].value, Sexp::Closure { .. } | Sexp::Function { .. }) {
            return Err(EvalError::Type(
                "cannot coerce type 'closure' to vector of type 'integer'".into(),
            ));
        }
        if matches!(args[0].value, Sexp::Factor { .. }) {
            return Err(EvalError::Domain(
                "argument 'x' must be an integer vector".into(),
            ));
        }
        let input = EvaluatedArgument {
            name: None,
            value: args[0].value.clone(),
        };
        let Sexp::Integer(values) = self.call_numeric_coercion("as.integer", &[input])? else {
            return Err(EvalError::Domain(
                "argument 'x' must be an integer vector".into(),
            ));
        };
        let multiple = as_logical(&args[1].value)
            .ok_or_else(|| EvalError::Domain("argument 'multiple' must be TRUE or FALSE".into()))?;
        let pairs = as_logical(&args[2].value).ok_or_else(|| {
            EvalError::Domain("argument 'allow_surrogate_pairs' must be TRUE or FALSE".into())
        })?;
        if multiple {
            if pairs {
                self.record_parse_warnings(&["allow_surrogate_pairs = TRUE is incompatible with multiple = TRUE and will be ignored".into()])?;
            }
            return Ok(Sexp::String(
                values
                    .into_iter()
                    .map(|value| {
                        value.and_then(codepoint).map(|value| {
                            if value == '\0' {
                                String::new()
                            } else {
                                value.to_string()
                            }
                        })
                    })
                    .collect(),
            ));
        }
        let mut text = String::new();
        let mut values = values.into_iter();
        while let Some(value) = values.next() {
            let character = match value {
                Some(high @ 0xD800..=0xDBFF) if pairs => match values.next() {
                    Some(Some(low @ 0xDC00..=0xDFFF)) => {
                        codepoint(0x10000 + (high - 0xD800) * 1024 + low - 0xDC00)
                    }
                    _ => None,
                },
                Some(value) => codepoint(value),
                None => None,
            };
            let Some(character) = character else {
                return Ok(Sexp::String(vec![None]));
            };
            if character != '\0' {
                text.push(character);
            }
        }
        Ok(Sexp::String(vec![Some(text)]))
    }
}

fn codepoint(value: i32) -> Option<char> {
    u32::try_from(value).ok().and_then(char::from_u32)
}
