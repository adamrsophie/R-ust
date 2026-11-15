use crate::{ordinary_values, EvalError, EvaluatedArgument, Sexp, SourceFault};

// R 4.6.0 src/main/coerce.c: do_isvector, including the names-only attribute rule.
pub(super) fn test(args: &[EvaluatedArgument], call: &Sexp) -> Result<Sexp, EvalError> {
    let Some([mode]) = args[1].value.character_values() else {
        return Err(SourceFault::type_error(
            "invalid 'mode' argument".into(),
            call,
        ));
    };
    let mode = mode.as_deref().unwrap_or("NA");
    let mode = if mode == "name" { "symbol" } else { mode };
    let projection = ordinary_values::read(&args[0].value)?;
    let value = projection.as_ref();
    let matches = match mode {
        "any" => matches!(
            value,
            Sexp::Logical(_)
                | Sexp::Integer(_)
                | Sexp::Real(_)
                | Sexp::String(_)
                | Sexp::AttributedCharacter { .. }
                | Sexp::NamedList { .. }
                | Sexp::List(_)
                | Sexp::Vector(_)
                | Sexp::Expression { .. }
        ),
        "numeric" => matches!(value, Sexp::Integer(_) | Sexp::Real(_)),
        _ => mode == crate::metadata::typeof_value(value),
    };
    let attributes_ok = match value {
        Sexp::AttributedCharacter { attributes, .. } | Sexp::Closure { attributes, .. } => {
            attributes
                .iter()
                .all(|attribute| attribute.name.as_deref() == Some("names"))
        }
        Sexp::ClassedList { .. }
        | Sexp::NamedList { class: Some(_), .. }
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. } => false,
        _ => true,
    };
    Ok(Sexp::Logical(vec![Some(matches && attributes_ok)]))
}
