use super::*;

pub(super) fn class_value(value: &Sexp) -> Sexp {
    let values = match value {
        Sexp::Symbol(_) | Sexp::MissingValue => vec!["name".to_string()],
        Sexp::PairList(_) => vec!["pairlist".to_string()],
        Sexp::Call { function, .. } => vec![match function.as_ref() {
            Sexp::Symbol(name)
                if matches!(
                    name.as_str(),
                    "if" | "for" | "while" | "(" | "{" | "=" | "<-"
                ) =>
            {
                name.clone()
            }
            _ => "call".to_string(),
        }],
        Sexp::Nil => vec!["NULL".to_string()],
        Sexp::Logical(_) => vec!["logical".to_string()],
        Sexp::Integer(_) => vec!["integer".to_string()],
        Sexp::Real(_) => vec!["numeric".to_string()],
        Sexp::String(_) => vec!["character".to_string()],
        Sexp::Factor { ordered: true, .. } => {
            vec!["ordered".to_string(), "factor".to_string()]
        }
        Sexp::Factor { .. } => vec!["factor".to_string()],
        Sexp::DataFrame { .. } => vec!["data.frame".to_string()],
        Sexp::Matrix { .. } => vec!["matrix".to_string(), "array".to_string()],
        Sexp::List(_) | Sexp::Dots(_) | Sexp::Vector(_) => vec!["list".to_string()],
        Sexp::Closure { .. } | Sexp::Function { .. } | Sexp::Primitive(_) => {
            vec!["function".to_string()]
        }
        _ => vec![typeof_value(value).to_string()],
    };
    Sexp::String(values.into_iter().map(Some).collect())
}

pub(super) fn typeof_value(value: &Sexp) -> &'static str {
    match value {
        Sexp::Nil => "NULL",
        Sexp::Logical(_) => "logical",
        Sexp::Integer(_) | Sexp::Factor { .. } => "integer",
        Sexp::Real(_) => "double",
        Sexp::String(_) => "character",
        Sexp::DataFrame { .. } | Sexp::List(_) | Sexp::Vector(_) => "list",
        Sexp::Matrix { data, .. } => typeof_value(data),
        Sexp::Dots(_) => "...",
        Sexp::Promise { .. } => "promise",
        Sexp::Closure { .. } | Sexp::Function { .. } => "closure",
        Sexp::Primitive(id) => match id {
            r_ust_sexp::PrimitiveId::Sum
            | r_ust_sexp::PrimitiveId::Length
            | r_ust_sexp::PrimitiveId::Invisible
            | r_ust_sexp::PrimitiveId::Add
            | r_ust_sexp::PrimitiveId::Subtract
            | r_ust_sexp::PrimitiveId::Multiply
            | r_ust_sexp::PrimitiveId::Divide
            | r_ust_sexp::PrimitiveId::Power
            | r_ust_sexp::PrimitiveId::Modulo
            | r_ust_sexp::PrimitiveId::IntegerDivide
            | r_ust_sexp::PrimitiveId::Colon => "builtin",
            r_ust_sexp::PrimitiveId::Return | r_ust_sexp::PrimitiveId::Missing => "special",
        },
        Sexp::Symbol(_) | Sexp::MissingValue => "symbol",
        Sexp::PairList(_) => "pairlist",
        _ => "language",
    }
}

pub(super) fn mode_value(value: &Sexp) -> &'static str {
    match value {
        Sexp::Symbol(_) | Sexp::MissingValue => "name",
        Sexp::Call { .. } => "call",
        Sexp::Nil => "NULL",
        Sexp::Logical(_) => "logical",
        Sexp::Integer(_) | Sexp::Real(_) | Sexp::Factor { .. } => "numeric",
        Sexp::String(_) => "character",
        Sexp::DataFrame { .. } | Sexp::List(_) | Sexp::Vector(_) => "list",
        Sexp::Matrix { data, .. } => mode_value(data),
        Sexp::Closure { .. } | Sexp::Function { .. } | Sexp::Primitive(_) => "function",
        _ => typeof_value(value),
    }
}
