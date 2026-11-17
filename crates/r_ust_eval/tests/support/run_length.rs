#[path = "warning_payload.rs"]
mod warning_payload;

use r_ust_eval::{EvalError, OutputEvent};
use r_ust_sexp::{is_na_real, AggregateVector, Sexp};

pub fn encode(value: &Sexp) -> String {
    if let Sexp::Managed(value) = value {
        return encode(&value.snapshot().expect("valid fixture transport"));
    }
    if let Some(view) = value.ordinary_list() {
        return format!(
            "list:{}",
            view.elements
                .iter()
                .map(|v| format!("{}={}", v.name.as_deref().unwrap_or(""), encode(&v.value)))
                .collect::<Vec<_>>()
                .join(";")
        );
    }
    let (kind, values) = match value {
        Sexp::Nil => return "null".into(),
        Sexp::AttributedAtomic { values, attributes } => {
            let class = attributes
                .iter()
                .find(|a| a.name.as_deref() == Some("class"))
                .and_then(|a| a.value.character_values());
            if class.is_some_and(|classes| classes.iter().any(|c| c.as_deref() == Some("factor"))) {
                let levels = attributes
                    .iter()
                    .find(|a| a.name.as_deref() == Some("levels"))
                    .expect("factor levels");
                return format!(
                    "factor:{}:{}",
                    encode(&values.clone().into_value()),
                    encode(&levels.value)
                );
            }
            return encode(&values.clone().into_value());
        }
        Sexp::AttributedCharacter { values, .. } => return encode(&Sexp::String(values.clone())),
        Sexp::Vector(values) => {
            return format!(
                "list:{}",
                values
                    .iter()
                    .map(|v| format!("={}", encode(v)))
                    .collect::<Vec<_>>()
                    .join(";")
            )
        }
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            attributes,
        } => {
            let names = attributes
                .iter()
                .find(|a| a.name.as_deref() == Some("names"))
                .and_then(|a| a.value.character_values());
            return format!(
                "list:{}",
                values
                    .iter()
                    .enumerate()
                    .map(|(i, value)| format!(
                        "{}={}",
                        names
                            .and_then(|names| names.get(i))
                            .and_then(|n| n.as_deref())
                            .unwrap_or(""),
                        encode(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(";")
            );
        }
        Sexp::Logical(values) => (
            "logical",
            values
                .iter()
                .map(|v| match v {
                    Some(true) => "TRUE".into(),
                    Some(false) => "FALSE".into(),
                    None => "NA".into(),
                })
                .collect::<Vec<String>>(),
        ),
        Sexp::Integer(values) => (
            "integer",
            values
                .iter()
                .map(|v| v.map_or("NA".into(), |v| v.to_string()))
                .collect(),
        ),
        Sexp::Real(values) => (
            "double",
            values
                .iter()
                .map(|v| {
                    if is_na_real(*v) {
                        "NA".into()
                    } else if v.is_nan() {
                        "NaN".into()
                    } else {
                        v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                    }
                })
                .collect(),
        ),
        Sexp::String(values) => (
            "character",
            values
                .iter()
                .map(|v| v.as_ref().map_or("NA".into(), |v| format!("{v:?}")))
                .collect(),
        ),
        Sexp::List(values) => {
            return format!(
                "list:{}",
                values
                    .iter()
                    .map(|v| format!("{}={}", v.name.as_deref().unwrap_or(""), encode(&v.value)))
                    .collect::<Vec<_>>()
                    .join(";")
            )
        }
        Sexp::Factor { codes, levels, .. } => {
            return format!(
                "factor:{}:{}",
                encode(&Sexp::Integer(
                    codes
                        .iter()
                        .map(|code| code.map(|code| i32::try_from(code).unwrap()))
                        .collect()
                )),
                encode(&Sexp::String(levels.iter().cloned().map(Some).collect()))
            )
        }
        other => panic!("unhandled fixture value {other:?}"),
    };
    format!("{kind}:{}", values.join(","))
}

pub fn error_payload(error: EvalError) -> String {
    match error {
        EvalError::Type(message)
        | EvalError::Domain(message)
        | EvalError::Arity(message)
        | EvalError::Stop { message, .. } => message,
        EvalError::UndefinedSymbol(name) => format!("object '{name}' not found"),
        other => panic!("unexpected error variant: {other:?}"),
    }
}

pub fn condition_events(events: Vec<OutputEvent>) -> Vec<OutputEvent> {
    warning_payload::conditions(events)
}

pub fn warning_events(messages: &str) -> Vec<OutputEvent> {
    if messages.is_empty() {
        return Vec::new();
    }
    messages
        .split('\u{1f}')
        .map(|message| {
            assert!(!message.is_empty(), "empty warning message");
            OutputEvent::Stderr(message.into())
        })
        .collect()
}
