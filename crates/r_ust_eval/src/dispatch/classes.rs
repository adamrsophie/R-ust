use crate::{EvalError, ListElement, Sexp};

pub(super) fn classes(value: &Sexp) -> Result<Sexp, EvalError> {
    let names: &[&str] = match value {
        Sexp::ListObject(value) => return classes(&value.value()?),
        Sexp::Char(_) => {
            return Err(EvalError::Domain(
                "cannot have attributes on a CHARSXP".into(),
            ));
        }
        Sexp::ClassedList { class, .. } => {
            let values = class
                .character_values()
                .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
            if !values.is_empty() {
                return Ok(class.as_ref().clone());
            }
            &["list"]
        }
        Sexp::Integer(_) => &["integer", "numeric"],
        Sexp::Real(_) => &["double", "numeric"],
        Sexp::Matrix { data, .. } => {
            let suffix: &[&str] = match crate::metadata::typeof_value(data) {
                "integer" => &["integer", "numeric"],
                "double" => &["double", "numeric"],
                "closure" | "builtin" | "special" => &["function"],
                "symbol" => &["name"],
                other => &[other],
            };
            let mut values = vec![Some("matrix".into()), Some("array".into())];
            values.extend(suffix.iter().map(|name| Some((*name).into())));
            return Ok(Sexp::String(values));
        }
        Sexp::Nil => &["NULL"],
        Sexp::Logical(_) => &["logical"],
        Sexp::String(_) | Sexp::AttributedCharacter { .. } => &["character"],
        Sexp::Factor { ordered: true, .. } => &["ordered", "factor"],
        Sexp::Factor { ordered: false, .. } => &["factor"],
        Sexp::DataFrame { .. } => &["data.frame"],
        Sexp::List(_) | Sexp::Vector(_) => &["list"],
        Sexp::PairList(_) | Sexp::PairListObject(_) => &["pairlist"],
        Sexp::Dots(_) => &["..."],
        Sexp::Environment(_) => &["environment"],
        Sexp::Closure { .. } | Sexp::Primitive(_) => &["function"],
        Sexp::Symbol(_) | Sexp::MissingValue | Sexp::MissingArg(_) => &["name"],
        Sexp::Promise { .. } => &["promise"],
        Sexp::If { .. } => &["if"],
        Sexp::For { .. } => &["for"],
        Sexp::While { .. } => &["while"],
        Sexp::Parenthesized(_) => &["("],
        Sexp::Block(_) => &["{"],
        Sexp::Assignment { operator, .. }
        | Sexp::Subassignment { operator, .. }
        | Sexp::MatrixSubassignment { operator, .. }
        | Sexp::ExtractAssignment { operator, .. }
        | Sexp::DollarAssignment { operator, .. }
        | Sexp::MemberAssignment { operator, .. }
        | Sexp::ReplacementAssignment { operator, .. }
        | Sexp::NamesAssignment { operator, .. } => {
            return classes(&Sexp::Call {
                function: Box::new(Sexp::Symbol(operator.as_str().into())),
                arguments: Vec::new(),
            });
        }
        Sexp::Call { function, .. } => match function.as_ref() {
            Sexp::Symbol(name)
                if matches!(
                    name.as_str(),
                    "if" | "for" | "while" | "(" | "{" | "=" | "<-"
                ) =>
            {
                return Ok(Sexp::String(vec![Some(name.clone())]));
            }
            _ => &["call"],
        },
        Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::ExtractArguments { .. }
        | Sexp::Dollar { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. } => &["call"],
    };
    Ok(Sexp::String(
        names.iter().map(|name| Some((*name).into())).collect(),
    ))
}

pub(super) fn suffix(full: &Sexp, index: usize) -> Result<Sexp, EvalError> {
    if index == 0 {
        return Ok(full.clone());
    }
    let values = full
        .character_values()
        .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
    Ok(Sexp::AttributedCharacter {
        values: values[index..].to_vec(),
        attributes: vec![ListElement {
            name: Some("previous".into()),
            value: full.clone(),
        }],
    })
}
