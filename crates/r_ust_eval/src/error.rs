use std::fmt;

use super::{ControlTransfer, QuitRequest};

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    Parse(String),
    UndefinedSymbol(String),
    UndefinedFunction(String),
    Type(String),
    Arity(String),
    Domain(String),
    Stop {
        message: String,
        call_requested: bool,
    },
    Control(String),
    Transfer(ControlTransfer),
    MissingArgument(String),
    Output(String),
    Input(String),
    Quit(QuitRequest),
    NotImplemented(&'static str),
    WithWarnings {
        error: Box<EvalError>,
        warnings: String,
    },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::UndefinedSymbol(name) => write!(f, "object '{name}' not found"),
            Self::UndefinedFunction(name) => write!(f, "could not find function \"{name}\""),
            Self::Type(message) => write!(f, "type error: {message}"),
            Self::Arity(message) => write!(f, "argument error: {message}"),
            Self::Domain(message) => write!(f, "domain error: {message}"),
            Self::Stop { message, .. } => f.write_str(message),
            Self::Control(message) => write!(f, "control-flow error: {message}"),
            Self::Transfer(transfer) => write!(f, "{}", transfer.inactive_message()),
            Self::MissingArgument(name) => {
                if name.is_empty() {
                    write!(f, "argument is missing, with no default")
                } else {
                    write!(f, "argument \"{name}\" is missing, with no default")
                }
            }
            Self::Output(message) => write!(f, "output error: {message}"),
            Self::Input(message) => write!(f, "input error: {message}"),
            Self::Quit(request) => {
                write!(f, "session exit requested with status {}", request.status)
            }
            Self::NotImplemented(feature) => write!(f, "{feature} is not implemented yet"),
            Self::WithWarnings { error, warnings } => {
                write!(
                    f,
                    "{error}\nIn addition: {}",
                    warnings.trim_end_matches('\n')
                )
            }
        }
    }
}

impl std::error::Error for EvalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::WithWarnings { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}
