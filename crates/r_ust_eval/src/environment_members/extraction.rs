use super::MemberKey;
use crate::{member_values, EvalError, EvaluatedArgument, RuntimeState, Sexp};
use member_values::NameMatching;

impl RuntimeState {
    pub(crate) fn read_extraction_arguments(
        &mut self,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let Some((object, arguments)) = arguments.split_first() else {
            return Err(EvalError::Domain("no index specified".into()));
        };
        // R removes only the first exact and drop arguments by their full tags.
        let mut exact = None;
        let mut drop_seen = false;
        let mut indices = Vec::new();
        for argument in arguments {
            match argument.name.as_deref() {
                Some("drop") if !drop_seen => drop_seen = true,
                Some("exact") if exact.is_none() => exact = Some(&argument.value),
                _ => indices.push(&argument.value),
            }
        }
        let matching = match exact {
            None | Some(Sexp::Nil) => NameMatching::Exact,
            Some(value) => {
                let value = crate::coercion::storage::read(value)?;
                match crate::internal::as_logical(value.as_ref()) {
                    Some(true) => NameMatching::Exact,
                    Some(false) => NameMatching::Partial,
                    None => NameMatching::Warn,
                }
            }
        };
        let Some(index) = indices.first() else {
            return Err(EvalError::Domain("no index specified".into()));
        };
        if matches!(object.value, Sexp::Nil) {
            return member_values::extract_value(&object.value, index);
        }
        if indices.len() != 1 {
            return Err(EvalError::Domain("incorrect number of subscripts".into()));
        }
        if matches!(object.value, Sexp::Environment(_)) {
            return self.read_member(&object.value, MemberKey::Index(index));
        }
        member_values::extract_matching(&object.value, index, matching, &mut |message, context| {
            self.signal_simple_warning(message, if context { call } else { &Sexp::Nil })
        })
    }
}
