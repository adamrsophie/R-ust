use super::{CallArgument, EvalError, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn render_unused_arguments(
        &mut self,
        arguments: &[&CallArgument],
    ) -> Result<String, EvalError> {
        // Snapshot immediate missingness before forcing any inner forwarded promise.
        let alist = arguments
            .iter()
            .any(|argument| matches!(argument.value, Sexp::MissingArg(_)));
        let mut parts = Vec::with_capacity(arguments.len());
        for argument in arguments {
            parts.push(self.deparse_argument(argument, 0)?);
        }
        let contents = parts.join(", ");
        let contents = if alist {
            format!("(alist({contents}))")
        } else {
            format!("({contents})")
        };
        let plural = if arguments.len() == 1 { "" } else { "s" };
        Ok(format!("unused argument{plural} {contents}"))
    }

    fn deparse_argument(
        &mut self,
        argument: &CallArgument,
        indent: usize,
    ) -> Result<String, EvalError> {
        let value = self.deparse_argument_expression(&argument.value, indent)?;
        Ok(match &argument.name {
            Some(name) => format!("{} = {value}", symbol(name)),
            None => value,
        })
    }

    fn deparse_argument_expression(
        &mut self,
        expression: &Sexp,
        indent: usize,
    ) -> Result<String, EvalError> {
        match expression {
            Sexp::Nil => Ok("NULL".into()),
            Sexp::MissingArg(_) | Sexp::MissingValue => Ok(String::new()),
            Sexp::Symbol(name) => Ok(symbol(name)),
            Sexp::Integer(values) if values.len() == 1 => {
                Ok(values[0].map_or_else(|| "NA".into(), |value| value.to_string()))
            }
            Sexp::Real(values) if values.len() == 1 => {
                Ok(super::real_literal::format_real(values[0], false))
            }
            Sexp::Logical(values) if values.len() == 1 => Ok(match values[0] {
                Some(true) => "TRUE",
                Some(false) => "FALSE",
                None => "NA",
            }
            .into()),
            Sexp::String(values) if values.len() == 1 => Ok(values[0]
                .as_ref()
                .map_or_else(|| "NA_character_".into(), |value| quoted(value, '"'))),
            Sexp::Promise { .. } => {
                // Direct ASTs already shed the supplied promise layer; this is an inner promise.
                let value = self.force_value(expression.clone())?;
                self.deparse_argument_expression(&value, indent)
            }
            Sexp::Call {
                function,
                arguments,
            } => {
                let head = self.deparse_argument_expression(function, indent)?;
                let mut parts = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    parts.push(self.deparse_argument(argument, indent)?);
                }
                Ok(format!("{head}({})", parts.join(", ")))
            }
            Sexp::Block(expressions) => {
                let mut text = String::from("{");
                for expression in expressions {
                    text.push('\n');
                    text.push_str(&" ".repeat(indent + 4));
                    text.push_str(&self.deparse_argument_expression(expression, indent + 4)?);
                }
                text.push('\n');
                text.push_str(&" ".repeat(indent));
                text.push('}');
                Ok(text)
            }
            Sexp::Parenthesized(expression) => Ok(format!(
                "({})",
                self.deparse_argument_expression(expression, indent)?
            )),
            Sexp::Break => Ok("break".into()),
            Sexp::Next => Ok("next".into()),
            // shortcut: scalar/call/block diagnostics only, extend with independent R deparse fixtures.
            Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::Vector(_)
            | Sexp::List(_)
            | Sexp::PairList(_)
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Closure { .. }
            | Sexp::Primitive(_)
            | Sexp::InternalCall { .. }
            | Sexp::Function { .. }
            | Sexp::If { .. }
            | Sexp::For { .. }
            | Sexp::While { .. }
            | Sexp::Repeat { .. }
            | Sexp::Subset { .. }
            | Sexp::Subset2D { .. }
            | Sexp::Extract { .. }
            | Sexp::Dollar { .. }
            | Sexp::Assignment { .. }
            | Sexp::SuperAssignment { .. }
            | Sexp::Subassignment { .. }
            | Sexp::MatrixSubassignment { .. }
            | Sexp::ExtractAssignment { .. }
            | Sexp::DollarAssignment { .. }
            | Sexp::NamesAssignment { .. }
            | Sexp::Binary { .. }
            | Sexp::Unary { .. } => Err(EvalError::NotImplemented(
                "unused argument expression deparsing",
            )),
        }
    }
}

fn symbol(name: &str) -> String {
    let first = name.chars().next();
    let valid = first.is_some_and(|c| c.is_alphabetic() || c == '.')
        && !(name.starts_with('.') && name.as_bytes().get(1).is_some_and(u8::is_ascii_digit))
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_')
        && !matches!(
            name,
            "if" | "else"
                | "repeat"
                | "while"
                | "function"
                | "for"
                | "in"
                | "next"
                | "break"
                | "TRUE"
                | "FALSE"
                | "NULL"
                | "Inf"
                | "NaN"
                | "NA"
        );
    if valid {
        name.into()
    } else {
        quoted(name, '`')
    }
}

fn quoted(text: &str, quote: char) -> String {
    let mut result = String::from(quote);
    for character in text.chars() {
        match character {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            c if c == quote => {
                result.push('\\');
                result.push(c);
            }
            c => result.push(c),
        }
    }
    result.push(quote);
    result
}
