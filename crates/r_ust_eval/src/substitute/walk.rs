use super::{canonical, environment::Lookup, promise_expression};
use crate::{CallArgument, EvalError, ListElement, Sexp, SourceFault};

pub(super) struct Substitution<'a> {
    lookup: Lookup,
    call: &'a Sexp,
}

impl<'a> Substitution<'a> {
    pub(super) fn new(lookup: Lookup, call: &'a Sexp) -> Self {
        Self { lookup, call }
    }

    pub(super) fn root(&self, value: &Sexp) -> Result<Sexp, EvalError> {
        if matches!(value, Sexp::Symbol(name) if name == "...") {
            return Ok(self
                .dots()?
                .into_iter()
                .next()
                .map(|element| element.value)
                .unwrap_or(Sexp::Nil));
        }
        self.value(value)
    }

    fn value(&self, value: &Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::Promise { state } => self.value(&canonical(&r_ust_sexp::bytecode_source(
                &state.expression()?,
            )?)?),
            Sexp::Symbol(name) => match self.lookup.get(name)? {
                Some(Sexp::Promise { state }) => promise_expression(&Sexp::Promise { state }),
                Some(Sexp::Dots(_)) => Err(self.dots_error()),
                Some(Sexp::MissingArg(_)) => Ok(Sexp::MissingValue),
                Some(value) => Ok(value),
                None => Ok(value.clone()),
            },
            Sexp::Call {
                function,
                arguments,
            } => self.language(function, arguments),
            Sexp::MissingArg(_) => Ok(Sexp::MissingValue),
            _ => Ok(value.clone()),
        }
    }

    fn language(&self, function: &Sexp, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let dots_head = matches!(function, Sexp::Symbol(name) if name == "...");
        let mut head = if dots_head {
            self.dots()?
        } else {
            vec![ListElement {
                name: None,
                value: self.value(function)?,
            }]
        };
        let language = !dots_head || self.lookup.get("...")?.is_none();
        for argument in arguments {
            if matches!(&argument.value, Sexp::Symbol(name) if name == "...") {
                head.extend(self.dots()?);
            } else {
                head.push(ListElement {
                    name: argument.name.clone(),
                    value: self.value(&argument.value)?,
                });
            }
        }
        if !language {
            return Ok(Sexp::pairlist(head));
        }
        let mut values = head.into_iter();
        let Some(function) = values.next() else {
            return Ok(Sexp::Nil);
        };
        Ok(Sexp::Call {
            function: Box::new(function.value),
            arguments: values
                .map(|value| CallArgument {
                    name: value.name,
                    value: value.value,
                })
                .collect(),
        })
    }

    fn dots(&self) -> Result<Vec<ListElement>, EvalError> {
        match self.lookup.get("...")? {
            None => Ok(vec![ListElement {
                name: None,
                value: Sexp::Symbol("...".into()),
            }]),
            Some(Sexp::Nil | Sexp::MissingArg(_) | Sexp::MissingValue) => Ok(Vec::new()),
            Some(Sexp::Dots(values)) => {
                let disabled = Self::new(Lookup::Disabled, self.call);
                let mut result = Vec::new();
                for value in values {
                    if matches!(&value.value, Sexp::Symbol(name) if name == "...") {
                        result.extend(disabled.dots()?);
                    } else {
                        result.push(ListElement {
                            name: value.name,
                            value: disabled.value(&canonical(&value.value)?)?,
                        });
                    }
                }
                Ok(result)
            }
            Some(_) => Err(self.dots_error()),
        }
    }

    fn dots_error(&self) -> EvalError {
        SourceFault::domain("'...' used in an incorrect context".into(), self.call)
    }
}
