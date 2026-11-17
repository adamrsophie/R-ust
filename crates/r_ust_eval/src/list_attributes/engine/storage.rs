use super::*;
use crate::{EvaluatedArgument, RuntimeState};
use r_ust_sexp::SexpType;

impl RuntimeState {
    pub(crate) fn assign_storage_mode(
        &mut self,
        object: &Sexp,
        replacement: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let replacement = payload(replacement)?;
        let name = replacement
            .character_values()
            .and_then(|v| v.first())
            .and_then(Option::as_deref)
            .ok_or_else(|| EvalError::Type("'value' must be non-null character string".into()))?;
        let kind = match name {
            "integer" => SexpType::Integer,
            "double" => SexpType::Real,
            "logical" => SexpType::Logical,
            "character" => SexpType::String,
            "list" => SexpType::Vector,
            "raw" => SexpType::Raw,
            "real" => {
                return Err(EvalError::Domain(
                    "use of 'real' is defunct: use 'double' instead".into(),
                ))
            }
            "single" => {
                return Err(EvalError::Domain(
                    "use of 'single' is defunct: use mode<- instead".into(),
                ))
            }
            "expression" | "complex" | "pairlist" => {
                return Err(EvalError::NotImplemented(
                    "storage mode conversion for this represented role",
                ))
            }
            _ => return Err(EvalError::Domain("invalid value".into())),
        };
        if object.sexptype() == kind {
            return Ok(object.clone());
        }
        let attrs = stored(object)?;
        if find(&attrs, "class")
            .map(payload)
            .transpose()?
            .as_ref()
            .and_then(Sexp::character_values)
            .is_some_and(|classes| {
                classes
                    .iter()
                    .any(|class| class.as_deref() == Some("factor"))
            })
        {
            return Err(EvalError::Domain(
                "invalid to change the storage mode of a factor".into(),
            ));
        }
        let plain = payload(object)?;
        let call = self.source_current_call();
        let converted = match kind {
            SexpType::Integer => self.call_numeric_coercion(
                "as.integer",
                &[EvaluatedArgument {
                    name: None,
                    value: plain,
                }],
            )?,
            SexpType::Vector => Sexp::Vector(crate::sequence_elements(&plain)?),
            SexpType::Raw => Sexp::Raw(self.raw_values_with_warning(&plain, &call)?),
            SexpType::Real => self.call_numeric_coercion(
                "as.double",
                &[EvaluatedArgument {
                    name: None,
                    value: plain,
                }],
            )?,
            SexpType::Logical => Sexp::Logical(crate::coerce_logical_values(&plain)?),
            SexpType::String => Sexp::String(crate::string_values(&plain)?),
            _ => return Err(EvalError::NotImplemented("non-atomic storage mode")),
        };
        let mut description = description(&converted)?
            .ok_or(EvalError::NotImplemented("converted storage attributes"))?;
        *attributes_mut(&mut description) = attrs;
        restore::value(&converted, description)
    }
}
