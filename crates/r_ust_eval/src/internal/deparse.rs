use crate::language_print::{format_deparse, Profile, WARN};
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};

impl RuntimeState {
    pub(super) fn internal_deparse(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let cutoff = if matches!(arguments[1].value, Sexp::Nil) {
            Some(60)
        } else {
            self.scalar_integer_with_warning(&arguments[1].value)?
        };
        let cutoff = match cutoff {
            Some(cutoff @ 20..=500) => usize::try_from(cutoff)
                .map_err(|_| SourceFault::domain("invalid deparse cutoff".into(), &call))?,
            _ => {
                self.signal_simple_warning(
                    "invalid 'cutoff' value for 'deparse', using default",
                    &call,
                )?;
                60
            }
        };
        let backtick = if matches!(arguments[2].value, Sexp::Nil) {
            false
        } else {
            boolean(&arguments[2].value, &call)?
        };
        let control = if matches!(arguments[3].value, Sexp::Nil) {
            4
        } else {
            self.scalar_integer_with_warning(&arguments[3].value)?
                .unwrap_or(i32::MIN)
        };
        let control = u32::from_ne_bytes(control.to_ne_bytes());
        let lines = self
            .scalar_integer_with_warning(&arguments[4].value)?
            .filter(|n| *n > 0)
            .and_then(|n| usize::try_from(n).ok());
        let profile = Profile {
            cutoff,
            backtick,
            control,
            lines,
            utf8: matches!(
                crate::text_locale::current()?,
                crate::text_locale::Encoding::Utf8
            ),
        };
        let (lines, incomplete) = format_deparse(self, &arguments[0].value, profile)?;
        if incomplete && control & WARN != 0 {
            self.signal_simple_warning("deparse may be incomplete", &call)?;
        }
        Ok(Sexp::String(lines.into_iter().map(Some).collect()))
    }

    pub(crate) fn deparse_call_line(&mut self, value: &Sexp) -> Result<String, EvalError> {
        let profile = Profile {
            cutoff: 60,
            backtick: true,
            control: 1089,
            lines: Some(1),
            utf8: matches!(
                crate::text_locale::current()?,
                crate::text_locale::Encoding::Utf8
            ),
        };
        let (lines, _) = format_deparse(self, value, profile)?;
        Ok(lines.into_iter().next().unwrap_or_default())
    }
}

fn boolean(value: &Sexp, call: &Sexp) -> Result<bool, EvalError> {
    if matches!(
        value,
        Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::AttributedCharacter { .. }
            | Sexp::Matrix { .. }
            | Sexp::Factor { .. }
    ) {
        let length = crate::value_len(value)?;
        if length > 1 {
            return Err(SourceFault::domain(
                format!("'length = {length}' in coercion to 'logical(1)'"),
                call,
            ));
        }
    }
    super::as_logical(value)
        .ok_or_else(|| SourceFault::domain("NA in coercion to boolean".into(), call))
}
