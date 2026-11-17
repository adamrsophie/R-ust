use super::*;

pub(super) fn class_value(value: &Sexp) -> Result<Sexp, EvalError> {
    if value.sexptype() == r_ust_sexp::SexpType::Environment {
        if let Some(class) = list_attributes::engine::stored(value)?
            .into_iter()
            .find(|a| a.name.as_deref() == Some("class"))
        {
            return Ok(class.value);
        }
    }
    if list_attributes::engine::description(value)?.is_some() {
        let class = list_attributes::engine::get(value, "class")?;
        if !matches!(class, Sexp::Nil) {
            return Ok(class);
        }
        let dim = list_attributes::engine::get(value, "dim")?;
        if !matches!(dim, Sexp::Nil) {
            let dims = list_attributes::engine::payload(&dim)?;
            let classes = if value_len(&dims)? == 2 {
                vec![Some("matrix".into()), Some("array".into())]
            } else {
                vec![Some("array".into())]
            };
            return Ok(Sexp::String(classes));
        }
        if matches!(
            value,
            Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. } | Sexp::Managed(_)
        ) {
            let name = match value.sexptype() {
                r_ust_sexp::SexpType::Real => "numeric",
                r_ust_sexp::SexpType::Closure => "function",
                _ => typeof_value(value),
            };
            return Ok(Sexp::String(vec![Some(name.into())]));
        }
    }
    if let Some(class) = crate::closure_attributes::explicit_class(value) {
        return Ok(class.clone());
    }
    if matches!(value, Sexp::Char(_)) {
        return Err(EvalError::Domain(
            "cannot have attributes on a CHARSXP".into(),
        ));
    }
    let projection = ordinary_values::read(value)?;
    let value = projection.as_ref();
    if let Some(view) = value.ordinary_list() {
        return Ok(view
            .class
            .cloned()
            .unwrap_or_else(|| Sexp::String(vec![Some("list".into())])));
    }
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
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => vec!["character".to_string()],
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
    Ok(Sexp::String(values.into_iter().map(Some).collect()))
}

pub(super) fn typeof_value(value: &Sexp) -> &'static str {
    match value {
        Sexp::Managed(_) | Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. } => {
            attribute_type(value.sexptype())
        }
        Sexp::Environment(_) => "environment",
        Sexp::Bytecode(_) => "bytecode",
        Sexp::Expression { .. } => "expression",
        Sexp::Nil => "NULL",
        Sexp::Logical(_) => "logical",
        Sexp::Integer(_) | Sexp::Factor { .. } => "integer",
        Sexp::Real(_) => "double",
        Sexp::Raw(_) => "raw",
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => "character",
        Sexp::Char(_) => "char",
        Sexp::DataFrame { .. }
        | Sexp::NamedList { .. }
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::ListObject(_)
        | Sexp::Vector(_) => "list",
        Sexp::Matrix { data, .. } => typeof_value(data),
        Sexp::Dots(_) => "...",
        Sexp::Promise { .. } => "promise",
        Sexp::Closure { .. } | Sexp::Function { .. } => "closure",
        Sexp::Primitive(id) => {
            if id.sexptype() == r_ust_sexp::SexpType::Special {
                "special"
            } else {
                "builtin"
            }
        }
        Sexp::Symbol(_) | Sexp::MissingValue => "symbol",
        Sexp::PairList(_) | Sexp::PairListObject(_) => "pairlist",
        _ => "language",
    }
}

pub(super) fn mode_value(value: &Sexp) -> &'static str {
    if matches!(
        value,
        Sexp::Managed(_) | Sexp::AttributedAtomic { .. } | Sexp::AttributedAggregate { .. }
    ) {
        return match value.sexptype() {
            r_ust_sexp::SexpType::Integer | r_ust_sexp::SexpType::Real => "numeric",
            r_ust_sexp::SexpType::Closure
            | r_ust_sexp::SexpType::Builtin
            | r_ust_sexp::SexpType::Special => "function",
            r_ust_sexp::SexpType::Language => "call",
            r_ust_sexp::SexpType::Symbol => "name",
            _ => typeof_value(value),
        };
    }
    match value {
        Sexp::Symbol(_) | Sexp::MissingValue => "name",
        Sexp::Call { .. } => "call",
        Sexp::Nil => "NULL",
        Sexp::Logical(_) => "logical",
        Sexp::Integer(_) | Sexp::Real(_) | Sexp::Factor { .. } => "numeric",
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => "character",
        Sexp::Char(_) => "char",
        Sexp::DataFrame { .. }
        | Sexp::NamedList { .. }
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::ListObject(_)
        | Sexp::Vector(_) => "list",
        Sexp::Matrix { data, .. } => mode_value(data),
        Sexp::Closure { .. } | Sexp::Function { .. } | Sexp::Primitive(_) => "function",
        _ => typeof_value(value),
    }
}

fn attribute_type(kind: r_ust_sexp::SexpType) -> &'static str {
    use r_ust_sexp::SexpType as T;
    match kind {
        T::Nil => "NULL",
        T::Symbol => "symbol",
        T::List => "pairlist",
        T::Closure => "closure",
        T::Environment => "environment",
        T::Promise => "promise",
        T::Language => "language",
        T::Special => "special",
        T::Builtin => "builtin",
        T::Char => "char",
        T::Logical => "logical",
        T::Integer => "integer",
        T::Real => "double",
        T::Complex => "complex",
        T::String => "character",
        T::Dots => "...",
        T::Any => "any",
        T::Vector => "list",
        T::Expression => "expression",
        T::Bytecode => "bytecode",
        T::ExternalPtr => "externalptr",
        T::WeakRef => "weakref",
        T::Raw => "raw",
        T::Object => "S4",
        T::New => "new",
        T::Free => "free",
        T::Function => "function",
    }
}
