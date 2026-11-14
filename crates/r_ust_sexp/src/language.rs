mod assignments;

use crate::{CallArgument, ListElement, Sexp, UnaryOp};

use super::language_types::LanguageError;

pub fn canonical_language(value: &Sexp) -> Result<Sexp, LanguageError> {
    Canonicalizer {
        runtime_leaves: RuntimeLeaves::Reject,
    }
    .language(value)
}

pub fn canonical_actual_call(value: &Sexp) -> Result<Sexp, LanguageError> {
    Canonicalizer {
        runtime_leaves: RuntimeLeaves::Preserve,
    }
    .language(value)
}

enum RuntimeLeaves {
    Reject,
    Preserve,
}

struct Canonicalizer {
    runtime_leaves: RuntimeLeaves,
}

impl Canonicalizer {
    fn language(&self, value: &Sexp) -> Result<Sexp, LanguageError> {
        let result = match value {
            Sexp::Nil
            | Sexp::Symbol(_)
            | Sexp::Logical(_)
            | Sexp::Integer(_)
            | Sexp::Real(_)
            | Sexp::String(_)
            | Sexp::MissingValue => value.clone(),
            Sexp::MissingArg(_) => Sexp::MissingValue,
            Sexp::PairList(elements) => Sexp::pairlist(
                elements
                    .as_slice()
                    .iter()
                    .map(|element| {
                        Ok(ListElement {
                            name: element.name.clone(),
                            value: self.language(&element.value)?,
                        })
                    })
                    .collect::<Result<_, LanguageError>>()?,
            ),
            Sexp::Call {
                function,
                arguments,
            } => {
                let head = match function.as_ref() {
                    Sexp::String(values) => match values.as_slice() {
                        [Some(name)] => Sexp::Symbol(name.clone()),
                        _ => return Err(LanguageError::RuntimeValue),
                    },
                    other => self.language(other)?,
                };
                Sexp::Call {
                    function: Box::new(head),
                    arguments: arguments
                        .iter()
                        .map(|argument| {
                            Ok(CallArgument {
                                name: argument.name.clone(),
                                value: self.language(&argument.value)?,
                            })
                        })
                        .collect::<Result<_, LanguageError>>()?,
                }
            }
            Sexp::Binary { op, left, right } => call(
                super::language_types::binary_name(*op),
                vec![self.language(left)?, self.language(right)?],
            ),
            Sexp::Unary { op, value } => call(
                match op {
                    UnaryOp::Plus => "+",
                    UnaryOp::Minus => "-",
                    UnaryOp::Not => "!",
                },
                vec![self.language(value)?],
            ),
            Sexp::Parenthesized(value) => call("(", vec![self.language(value)?]),
            Sexp::Block(values) => call(
                "{",
                values
                    .iter()
                    .map(|value| self.language(value))
                    .collect::<Result<_, _>>()?,
            ),
            Sexp::Function { parameters, body } => {
                let formals = parameters
                    .iter()
                    .map(|parameter| {
                        Ok(ListElement {
                            name: Some(parameter.name.clone()),
                            value: self.optional(parameter.default.as_ref())?,
                        })
                    })
                    .collect::<Result<_, LanguageError>>()?;
                call(
                    "function",
                    vec![Sexp::pairlist(formals), self.language(body)?, Sexp::Nil],
                )
            }
            Sexp::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut values = vec![self.language(condition)?, self.language(then_branch)?];
                if let Some(value) = else_branch {
                    values.push(self.language(value)?);
                }
                call("if", values)
            }
            Sexp::For {
                variable,
                sequence,
                body,
            } => call(
                "for",
                vec![
                    Sexp::Symbol(variable.clone()),
                    self.language(sequence)?,
                    self.language(body)?,
                ],
            ),
            Sexp::While { condition, body } => call(
                "while",
                vec![self.language(condition)?, self.language(body)?],
            ),
            Sexp::Repeat { body } => call("repeat", vec![self.language(body)?]),
            Sexp::Break => call("break", vec![]),
            Sexp::Next => call("next", vec![]),
            Sexp::Subset { value, index } => call(
                "[",
                vec![self.language(value)?, self.optional(index.as_deref())?],
            ),
            Sexp::Subset2D { value, row, col } => call(
                "[",
                vec![
                    self.language(value)?,
                    self.optional(row.as_deref())?,
                    self.optional(col.as_deref())?,
                ],
            ),
            Sexp::ExtractArguments { value, arguments } => {
                let mut canonical = vec![CallArgument::positional(self.language(value)?)];
                for argument in arguments {
                    canonical.push(CallArgument {
                        name: argument.name.clone(),
                        value: self.language(&argument.value)?,
                    });
                }
                Sexp::Call {
                    function: Box::new(Sexp::Symbol("[[".into())),
                    arguments: canonical,
                }
            }
            Sexp::Extract { value, index } => {
                call("[[", vec![self.language(value)?, self.language(index)?])
            }
            Sexp::Dollar { value, name } => call("$", vec![self.language(value)?, name.to_sexp()]),
            Sexp::Assignment { .. }
            | Sexp::SuperAssignment { .. }
            | Sexp::Subassignment { .. }
            | Sexp::MatrixSubassignment { .. }
            | Sexp::ExtractAssignment { .. }
            | Sexp::DollarAssignment { .. }
            | Sexp::MemberAssignment { .. }
            | Sexp::ReplacementAssignment { .. }
            | Sexp::NamesAssignment { .. } => assignments::canonical_assignment(value, self)?,
            Sexp::InternalCall { .. } => return Err(LanguageError::InternalCall),
            Sexp::AttributedCharacter { .. }
            | Sexp::Environment(_)
            | Sexp::Char(_)
            | Sexp::ListObject(_)
            | Sexp::PairListObject(_)
            | Sexp::Primitive(_)
            | Sexp::Vector(_)
            | Sexp::ClassedList { .. }
            | Sexp::List(_)
            | Sexp::Factor { .. }
            | Sexp::DataFrame { .. }
            | Sexp::Matrix { .. }
            | Sexp::Dots(_)
            | Sexp::Promise { .. }
            | Sexp::Closure { .. } => match self.runtime_leaves {
                RuntimeLeaves::Reject => return Err(LanguageError::RuntimeValue),
                RuntimeLeaves::Preserve => value.clone(),
            },
        };
        Ok(result)
    }

    fn optional(&self, value: Option<&Sexp>) -> Result<Sexp, LanguageError> {
        match value {
            Some(value) => self.language(value),
            None => Ok(Sexp::MissingValue),
        }
    }
}

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}
