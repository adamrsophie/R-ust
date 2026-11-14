use r_ust_sexp::PrimitiveId;

use crate::{EvalError, EvaluatedArgument, Sexp};

pub(super) fn lookup(arguments: &[EvaluatedArgument], call: &Sexp) -> Result<Sexp, EvalError> {
    let [argument] = arguments else {
        return Err(crate::SourceFault::arity(
            format!(
                "{} arguments passed to '.Primitive' which requires 1",
                arguments.len()
            ),
            call,
        ));
    };
    let (Sexp::String(values) | Sexp::AttributedCharacter { values, .. }) = &argument.value else {
        return Err(crate::SourceFault::type_error(
            "string argument required".into(),
            call,
        ));
    };
    let [name] = values.as_slice() else {
        return Err(crate::SourceFault::type_error(
            "string argument required".into(),
            call,
        ));
    };
    let name = name.as_deref().unwrap_or("NA");
    if let Some(id) = PrimitiveId::from_name(name) {
        return Ok(Sexp::Primitive(id));
    }
    if super::names::NAMES.contains(&name) {
        return Err(EvalError::NotImplemented("primitive catalogue entry"));
    }
    Err(crate::SourceFault::domain(
        "no such primitive function".into(),
        call,
    ))
}
