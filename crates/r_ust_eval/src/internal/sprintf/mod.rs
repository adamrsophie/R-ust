mod coercion;
mod hex;
mod integer;
mod padding;
mod real;
mod spec;

use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp, SourceFault};
use spec::{Conversion, Spec};

pub(crate) fn deparse_real(value: f64, hexadecimal: bool) -> Result<String, EvalError> {
    let format = if hexadecimal { "%a" } else { "%.17g" };
    let spec = Spec::parse(format.into()).map_err(EvalError::Domain)?;
    real::render(&spec, value).map_err(EvalError::Domain)
}

struct Formatting<'a> {
    runtime: &'a mut RuntimeState,
    call: Sexp,
    formats: Vec<Option<String>>,
    arguments: Vec<Sexp>,
    lengths: Vec<usize>,
    used: Vec<bool>,
    maximum: usize,
}

impl RuntimeState {
    pub(super) fn internal_sprintf(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let Some(first) = arguments.first() else {
            return Err(SourceFault::arity("'fmt' is missing".into(), &call));
        };
        let formats = first
            .value
            .character_values()
            .ok_or_else(|| SourceFault::domain("'fmt' is not a character vector".into(), &call))?
            .to_vec();
        if formats.is_empty() {
            return Ok(Sexp::String(Vec::new()));
        }
        let count = arguments.len() - 1;
        if count >= 100 {
            return Err(SourceFault::domain(
                "only 100 arguments are allowed".into(),
                &call,
            ));
        }
        let mut values = Vec::with_capacity(count);
        let mut lengths = Vec::with_capacity(count);
        for (index, argument) in arguments[1..].iter().enumerate() {
            let value = crate::ordinary_values::read(&argument.value)?.into_owned();
            let kind = match value.sexptype() {
                r_ust_sexp::SexpType::Language => Some("language"),
                r_ust_sexp::SexpType::Symbol => Some("symbol"),
                _ => None,
            };
            if let Some(kind) = kind {
                return Err(SourceFault::domain(
                    format!("invalid type of argument[{}]: '{kind}'", index + 1),
                    &call,
                ));
            }
            let length = crate::value_len(&value)?;
            if length == 0 {
                return Ok(Sexp::String(Vec::new()));
            }
            values.push(value);
            lengths.push(length);
        }
        let mut formatting = Formatting {
            runtime: self,
            call,
            formats,
            arguments: values,
            lengths,
            used: vec![false; count],
            maximum: 0,
        };
        formatting.recycling()?;
        formatting.run()
    }
}

impl Formatting<'_> {
    fn error(&self, message: String) -> EvalError {
        SourceFault::domain(message, &self.call)
    }

    fn recycling(&mut self) -> Result<(), EvalError> {
        self.maximum = self
            .lengths
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .max(self.formats.len());
        if self.maximum % self.formats.len() != 0
            || self.lengths.iter().any(|length| self.maximum % length != 0)
        {
            return Err(self.error("arguments cannot be recycled to the same length".into()));
        }
        Ok(())
    }

    fn run(&mut self) -> Result<Sexp, EvalError> {
        let mut result = Vec::new();
        let mut row = 0;
        while row < self.maximum {
            result.push(Some(self.row(row)?));
            row += 1;
        }
        let unused = self.used.iter().filter(|used| !**used).count();
        if unused > 0 {
            let count = if unused == 1 {
                "one argument".into()
            } else {
                format!("{unused} arguments")
            };
            let mut message = format!("{count} not used by format");
            if self.formats.len() == 1 {
                message.push_str(&format!(
                    " '{}'",
                    self.formats[0].as_deref().unwrap_or("NA")
                ));
            }
            self.runtime.signal_simple_warning(&message, &self.call)?;
        }
        Ok(Sexp::String(result))
    }

    fn row(&mut self, row: usize) -> Result<String, EvalError> {
        let format = self.formats[row % self.formats.len()]
            .clone()
            .unwrap_or_else(|| "NA".into());
        if format.len() > padding::MAX_LENGTH {
            return Err(self.error(format!(
                "'fmt' length exceeds maximal format length {}",
                padding::MAX_LENGTH
            )));
        }
        let mut output = String::new();
        let mut remaining = format.as_str();
        let mut cursor = 0;
        while let Some(start) = remaining.find('%') {
            output.push_str(&remaining[..start]);
            remaining = &remaining[start..];
            if remaining.starts_with("%%") {
                output.push('%');
                remaining = &remaining[2..];
                continue;
            }
            let end = remaining[1..]
                .char_indices()
                .find(|(_, c)| "diosfeEgGxXaA".contains(*c))
                .map(|(offset, _)| offset + 2)
                .ok_or_else(|| {
                    self.error(format!("unrecognised format specification '{remaining}'"))
                })?;
            let conversion =
                Conversion::parse(&remaining[..end], &mut cursor, self.arguments.len())
                    .map_err(|e| self.error(e))?;
            let text = if let Some(index) = conversion.star {
                let width = self.star(index, row)?;
                conversion.text.replacen('*', &width.to_string(), 1)
            } else {
                conversion.text
            };
            let spec = Spec::parse(text).map_err(|e| self.error(e))?;
            output.push_str(&self.render(conversion.argument, &spec, row)?);
            remaining = &remaining[end..];
        }
        output.push_str(remaining);
        Ok(output)
    }

    fn render(&mut self, index: usize, spec: &Spec, row: usize) -> Result<String, EvalError> {
        self.used[index] = true;
        if row == 0 {
            self.coerce(index, spec.conversion)?;
        }
        if spec.conversion == 's' && spec.text != "%s" {
            let long = self.arguments[index]
                .character_values()
                .and_then(|values| values.get(row % values.len()))
                .and_then(|value| value.as_ref())
                .is_some_and(|value| value.len() > padding::MAX_LENGTH);
            if long {
                self.runtime.signal_simple_warning(
                    "likely truncation of character string to 8191 characters",
                    &self.call,
                )?;
            }
        }
        let value = &self.arguments[index];
        let invalid = |hint| self.error(spec.error(hint));
        let result = match value {
            Sexp::Logical(values) => {
                if !spec.valid || !"di".contains(spec.conversion) {
                    return Err(invalid("use format %d or %i for logical objects"));
                }
                integer::render(spec, values[row % values.len()].map(i32::from))
            }
            Sexp::Integer(values) => {
                if !spec.valid || !"dioxX".contains(spec.conversion) {
                    return Err(invalid(
                        "use format %d, %i, %o, %x or %X for integer objects",
                    ));
                }
                integer::render(spec, values[row % values.len()])
            }
            Sexp::Real(values) => {
                if !spec.valid || !"aAfeEgG".contains(spec.conversion) {
                    return Err(invalid("use format %f, %e, %g or %a for numeric objects"));
                }
                real::render(spec, values[row % values.len()])
            }
            value if value.character_values().is_some() => {
                if !spec.valid || spec.conversion != 's' {
                    return Err(invalid("use format %s for character objects"));
                }
                let values = value
                    .character_values()
                    .ok_or_else(|| self.error("unsupported type".into()))?;
                let value = values[row % values.len()].as_deref().unwrap_or("NA");
                let end = spec.precision.unwrap_or(value.len()).min(value.len());
                if !value.is_char_boundary(end) {
                    return Err(EvalError::NotImplemented(
                        "sprintf precision splitting UTF-8 string bytes",
                    ));
                }
                if spec.text == "%s" {
                    Ok(value.into())
                } else {
                    padding::string(spec, value)
                }
            }
            _ => return Err(self.error("unsupported type".into())),
        };
        result.map_err(|message| self.error(message))
    }
}
