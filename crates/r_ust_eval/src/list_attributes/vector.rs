use crate::{string_scalar, EvalError, EvaluatedArgument, Sexp};

pub(crate) fn as_vector(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let [value, mode] = args else {
        return Err(EvalError::Arity("as.vector requires two arguments".into()));
    };
    let mode = string_scalar(&mode.value, "mode")?;
    let projection = crate::ordinary_values::read(&value.value)?;
    if let Some(view) = projection.ordinary_list() {
        return match mode.as_str() {
            "any" | "list" => Ok(value.value.clone()),
            "pairlist" => Ok(crate::list_construction::to_pairlist(view)),
            _ => Err(EvalError::NotImplemented(
                "as.vector list coercion to this mode",
            )),
        };
    }
    match (projection.as_ref(), mode.as_str()) {
        (Sexp::Nil | Sexp::PairList(_), "pairlist") => Ok(value.value.clone()),
        (Sexp::Nil, "list") => Ok(Sexp::List(Vec::new())),
        (Sexp::PairList(values), "list") => {
            Ok(crate::list_construction::from_tags(values.as_slice()))
        }
        _ => Err(EvalError::NotImplemented("as.vector of this type and mode")),
    }
}
