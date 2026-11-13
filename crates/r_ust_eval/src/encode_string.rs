mod escape;

use crate::{EvalError, EvalFlow, RuntimeState, Sexp};

const CHOICES: [&str; 4] = ["left", "right", "centre", "none"];

impl RuntimeState {
    pub(crate) fn encode_string_wrapper(&mut self) -> Result<Sexp, EvalError> {
        let EvalFlow::Value(x) = self.eval_flow(&Sexp::Symbol("x".into()))?;
        let x = crate::string_values(&x)?;
        let EvalFlow::Value(justify) = self.eval_flow(&Sexp::Symbol("justify".into()))?;
        let justify = justification(&justify)?;
        let EvalFlow::Value(width) = self.eval_flow(&Sexp::Symbol("width".into()))?;
        let EvalFlow::Value(quote) = self.eval_flow(&Sexp::Symbol("quote".into()))?;
        let EvalFlow::Value(na) = self.eval_flow(&Sexp::Symbol("na.encode".into()))?;
        self.encode_string_values(&[
            Sexp::String(x),
            width,
            quote,
            Sexp::Integer(vec![Some(justify)]),
            na,
        ])
    }

    pub(crate) fn encode_string_values(&mut self, args: &[Sexp]) -> Result<Sexp, EvalError> {
        let Sexp::String(values) = &args[0] else {
            return Err(EvalError::Type(
                "a character vector argument expected".into(),
            ));
        };
        let width = self.scalar_integer_with_warning(&args[1])?;
        if width.is_some_and(|width| width < 0) {
            return Err(invalid("width"));
        }
        let Sexp::String(quotes) = &args[2] else {
            return Err(invalid("quote"));
        };
        let [quote] = quotes.as_slice() else {
            return Err(invalid("quote"));
        };
        let quote = quote.as_deref().unwrap_or("NA");
        if !quote.is_ascii() {
            return Err(EvalError::NotImplemented(
                "encodeString non-ASCII quote bytes",
            ));
        }
        if quote.len() > 1 {
            self.record_parse_warnings(&[
                "only the first character of 'quote' will be used".into()
            ])?;
        }
        let quote = quote.chars().next();
        let justify = self
            .scalar_integer_with_warning(&args[3])?
            .ok_or_else(|| invalid("justify"))?;
        if !(0..=3).contains(&justify) {
            return Err(invalid("justify"));
        }
        let na = crate::internal::as_logical(&args[4]).ok_or_else(|| invalid("na.encode"))?;
        let encoded = values
            .iter()
            .map(|value| match value {
                Some(value) => Some(escape::encode(value, quote)),
                None if na => Some(if quote.is_some() {
                    "NA".into()
                } else {
                    "<NA>".into()
                }),
                None => None,
            })
            .collect::<Vec<_>>();
        let width = match (justify, width) {
            (3, _) => 0,
            (_, Some(width)) => usize::try_from(width).map_err(|_| invalid("width"))?,
            (_, None) => encoded
                .iter()
                .zip(values)
                .filter_map(|(encoded, original)| {
                    encoded.as_ref().map(|value| {
                        escape::width(value)
                            + if original.is_none() && quote.is_some() {
                                2
                            } else {
                                0
                            }
                    })
                })
                .max()
                .unwrap_or(0),
        };
        let result = encoded
            .into_iter()
            .map(|value| {
                value.map(|value| {
                    let padding = width.saturating_sub(escape::width(&value));
                    let left = match justify {
                        1 => padding,
                        2 => padding / 2,
                        _ => 0,
                    };
                    format!(
                        "{}{}{}",
                        " ".repeat(left),
                        value,
                        " ".repeat(padding - left)
                    )
                })
            })
            .collect();
        Ok(Sexp::String(result))
    }
}

fn invalid(name: &str) -> EvalError {
    EvalError::Domain(format!("invalid '{name}' value"))
}

fn justification(value: &Sexp) -> Result<i32, EvalError> {
    if matches!(value, Sexp::Nil) {
        return Ok(0);
    }
    let Sexp::String(values) = value else {
        return Err(EvalError::Domain(
            "'arg' must be NULL or a character vector".into(),
        ));
    };
    if values
        .iter()
        .map(|v| v.as_deref())
        .eq(CHOICES.iter().copied().map(Some))
    {
        return Ok(0);
    }
    if values.len() != 1 {
        return Err(EvalError::Domain("'arg' must be of length 1".into()));
    }
    let choice = values
        .first()
        .and_then(|value| value.as_deref())
        .unwrap_or("");
    if !choice.is_empty() {
        if let Some(index) = CHOICES
            .iter()
            .position(|candidate| candidate.starts_with(choice))
        {
            return i32::try_from(index).map_err(|_| invalid("justify"));
        }
    }
    Err(EvalError::Domain(
        "'arg' should be one of “left”, “right”, “centre”, “none”".into(),
    ))
}
