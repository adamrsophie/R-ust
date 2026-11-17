// Adapted from GNU R coerce.c/deparse.c, GPL-2.0-or-later.
// Copyright R Core Team, R Foundation, Robert Gentleman and Ross Ihaka.
use crate::language_print::{format_deparse, Profile};
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};

impl RuntimeState {
    pub(crate) fn call_character_coercion(
        &mut self,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let Some(first) = arguments.first() else {
            return Ok(Sexp::String(Vec::new()));
        };
        if let Some(name) = first.name.as_deref().filter(|name| *name != "x") {
            return Err(SourceFault::arity(
                format!("supplied argument name '{name}' does not match 'x'"),
                call,
            ));
        }
        if let Some(value) = self.dispatch_internal_generic("as.character", arguments, call)? {
            return Ok(value);
        }
        self.character_coercion_value(&first.value)
    }

    fn character_coercion_value(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        let projection = crate::ordinary_values::read(value)?;
        let value = projection.as_ref();
        if let Sexp::Matrix { data, .. } = value {
            return self.character_coercion_value(data);
        }
        let elements = if let Some(view) = value.ordinary_list() {
            view.elements
                .iter()
                .map(|element| &element.value)
                .collect::<Vec<_>>()
        } else {
            match value {
                Sexp::Expression { elements, .. }
                | Sexp::DataFrame {
                    columns: elements, ..
                } => elements.iter().map(|element| &element.value).collect(),
                Sexp::Vector(elements) => elements.iter().collect(),
                _ => return Ok(Sexp::String(crate::string_values(value)?)),
            }
        };
        let mut result = Vec::with_capacity(elements.len());
        for element in elements {
            if crate::is_character_value(element) {
                let values = crate::string_values(element)?;
                if let [value] = values.as_slice() {
                    result.push(value.clone());
                    continue;
                }
            }
            let profile = Profile {
                cutoff: 500,
                backtick: true,
                control: 1024,
                lines: None,
                utf8: matches!(
                    crate::text_locale::current()?,
                    crate::text_locale::Encoding::Utf8
                ),
            };
            let (lines, _) = format_deparse(self, element, profile)?;
            result.push(Some(lines.join("\n")));
        }
        Ok(Sexp::String(result))
    }
}
