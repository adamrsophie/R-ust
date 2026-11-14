use r_ust_sexp::PrimitiveId;

use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn lookup(arguments: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let [argument] = arguments else {
        return Err(EvalError::Arity(format!(
            "{} arguments passed to '.Primitive' which requires 1",
            arguments.len()
        )));
    };
    let Sexp::String(values) = &argument.value else {
        return Err(EvalError::Type("string argument required".into()));
    };
    let [name] = values.as_slice() else {
        return Err(EvalError::Type("string argument required".into()));
    };
    let name = name.as_deref().unwrap_or("NA");
    if let Some(id) = PrimitiveId::from_name(name) {
        return Ok(Sexp::Primitive(id));
    }
    if super::names::NAMES.contains(&name) {
        return Err(EvalError::NotImplemented("primitive catalogue entry"));
    }
    Err(EvalError::Domain("no such primitive function".into()))
}
