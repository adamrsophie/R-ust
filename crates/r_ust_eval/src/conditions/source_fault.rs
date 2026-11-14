use crate::{EvalError, Sexp};
use r_ust_sexp::{canonical_actual_call, LanguageError, ListElement};

#[derive(Clone, Debug, PartialEq)]
pub struct SourceFault {
    pub(crate) original: Box<EvalError>,
    pub(crate) message: String,
    pub(crate) call: Result<Sexp, LanguageError>,
    missing: Option<MissingSite>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MissingSite {
    EvaluatedSymbol,
    FunctionBinding,
}

impl SourceFault {
    pub(crate) fn domain(message: String, call: &Sexp) -> EvalError {
        Self::simple(EvalError::Domain(message.clone()), message, call.clone())
    }

    pub(crate) fn arity(message: String, call: &Sexp) -> EvalError {
        Self::simple(EvalError::Arity(message.clone()), message, call.clone())
    }

    pub(crate) fn type_error(message: String, call: &Sexp) -> EvalError {
        Self::simple(EvalError::Type(message.clone()), message, call.clone())
    }

    pub(crate) fn undefined_symbol(name: &str, call: &Sexp) -> EvalError {
        let original = EvalError::UndefinedSymbol(name.into());
        let message = original.to_string();
        Self::simple(original, message, call.clone())
    }

    pub(crate) fn undefined_function(name: &str, call: &Sexp) -> EvalError {
        let original = EvalError::UndefinedFunction(name.into());
        let message = original.to_string();
        Self::simple(original, message, call.clone())
    }

    pub(crate) fn stop(message: String, requested: bool, call: &Sexp) -> EvalError {
        let original = EvalError::Stop {
            message: message.clone(),
            call_requested: requested,
        };
        Self::simple(original, message, call.clone())
    }

    fn simple(original: EvalError, message: String, call: Sexp) -> EvalError {
        EvalError::Fresh(Box::new(Self {
            original: Box::new(original),
            message,
            call: canonical_actual_call(&call),
            missing: None,
        }))
    }

    pub(crate) fn missing(name: String, call: Sexp, site: MissingSite) -> EvalError {
        let original = EvalError::MissingArgument(name);
        let message = original.to_string();
        EvalError::Fresh(Box::new(Self {
            original: Box::new(original),
            message,
            call: canonical_actual_call(&call),
            missing: Some(site),
        }))
    }

    pub(super) fn classes(&self) -> Vec<Option<String>> {
        let names = match self.missing {
            None => vec!["simpleError", "error", "condition"],
            Some(MissingSite::EvaluatedSymbol) => {
                vec!["evalError", "missingArgError", "error", "condition"]
            }
            Some(MissingSite::FunctionBinding) => {
                vec!["getMissingError", "missingArgError", "error", "condition"]
            }
        };
        names.into_iter().map(|name| Some(name.into())).collect()
    }

    pub(super) fn condition(&self, call: &Sexp) -> Option<Sexp> {
        self.missing?;
        Some(Sexp::ClassedList {
            elements: vec![
                ListElement {
                    name: Some("message".into()),
                    value: Sexp::String(vec![Some(self.message.clone())]),
                },
                ListElement {
                    name: Some("call".into()),
                    value: call.clone(),
                },
            ],
            class: Box::new(Sexp::String(self.classes())),
            class_first: false,
        })
    }
}
