use crate::{CallArgument, ListElement, Sexp, UnaryOp};

use super::language_types::LanguageError;

pub fn canonical_language(value: &Sexp) -> Result<Sexp, LanguageError> {
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
                        value: canonical_language(&element.value)?,
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
                other => canonical_language(other)?,
            };
            Sexp::Call {
                function: Box::new(head),
                arguments: arguments
                    .iter()
                    .map(|argument| {
                        Ok(CallArgument {
                            name: argument.name.clone(),
                            value: canonical_language(&argument.value)?,
                        })
                    })
                    .collect::<Result<_, LanguageError>>()?,
            }
        }
        Sexp::Binary { op, left, right } => call(
            super::language_types::binary_name(*op),
            vec![canonical_language(left)?, canonical_language(right)?],
        ),
        Sexp::Unary { op, value } => call(
            match op {
                UnaryOp::Plus => "+",
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
            },
            vec![canonical_language(value)?],
        ),
        Sexp::Parenthesized(value) => call("(", vec![canonical_language(value)?]),
        Sexp::Block(values) => call(
            "{",
            values
                .iter()
                .map(canonical_language)
                .collect::<Result<_, _>>()?,
        ),
        Sexp::Function { parameters, body } => {
            let formals = parameters
                .iter()
                .map(|parameter| {
                    Ok(ListElement {
                        name: Some(parameter.name.clone()),
                        value: optional(parameter.default.as_ref())?,
                    })
                })
                .collect::<Result<_, LanguageError>>()?;
            call(
                "function",
                vec![
                    Sexp::pairlist(formals),
                    canonical_language(body)?,
                    Sexp::Nil,
                ],
            )
        }
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut values = vec![
                canonical_language(condition)?,
                canonical_language(then_branch)?,
            ];
            if let Some(value) = else_branch {
                values.push(canonical_language(value)?);
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
                canonical_language(sequence)?,
                canonical_language(body)?,
            ],
        ),
        Sexp::While { condition, body } => call(
            "while",
            vec![canonical_language(condition)?, canonical_language(body)?],
        ),
        Sexp::Repeat { body } => call("repeat", vec![canonical_language(body)?]),
        Sexp::Break => call("break", vec![]),
        Sexp::Next => call("next", vec![]),
        Sexp::Subset { value, index } => call(
            "[",
            vec![canonical_language(value)?, optional(index.as_deref())?],
        ),
        Sexp::Subset2D { value, row, col } => call(
            "[",
            vec![
                canonical_language(value)?,
                optional(row.as_deref())?,
                optional(col.as_deref())?,
            ],
        ),
        Sexp::Extract { value, index } => call(
            "[[",
            vec![canonical_language(value)?, canonical_language(index)?],
        ),
        Sexp::Dollar { value, name } => call("$", vec![canonical_language(value)?, name.to_sexp()]),
        Sexp::Assignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![Sexp::Symbol(target.clone()), canonical_language(value)?],
        ),
        Sexp::SuperAssignment { target, value } => call(
            "<<-",
            vec![Sexp::Symbol(target.clone()), canonical_language(value)?],
        ),
        Sexp::Subassignment {
            operator,
            target,
            index,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[",
                    vec![Sexp::Symbol(target.clone()), optional(index.as_deref())?],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::MatrixSubassignment {
            operator,
            target,
            row,
            col,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[",
                    vec![
                        Sexp::Symbol(target.clone()),
                        optional(row.as_deref())?,
                        optional(col.as_deref())?,
                    ],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::ExtractAssignment {
            operator,
            target,
            index,
            value,
        } => call(
            operator.as_str(),
            vec![
                call(
                    "[[",
                    vec![Sexp::Symbol(target.clone()), canonical_language(index)?],
                ),
                canonical_language(value)?,
            ],
        ),
        Sexp::DollarAssignment {
            operator,
            target,
            name,
            value,
        } => call(
            operator.as_str(),
            vec![
                call("$", vec![Sexp::Symbol(target.clone()), name.to_sexp()]),
                canonical_language(value)?,
            ],
        ),
        Sexp::NamesAssignment {
            operator,
            target,
            value,
        } => call(
            operator.as_str(),
            vec![
                call("names", vec![Sexp::Symbol(target.clone())]),
                canonical_language(value)?,
            ],
        ),
        Sexp::InternalCall { .. } => return Err(LanguageError::InternalCall),
        Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. } => return Err(LanguageError::RuntimeValue),
    };
    Ok(result)
}

fn optional(value: Option<&Sexp>) -> Result<Sexp, LanguageError> {
    match value {
        Some(value) => canonical_language(value),
        None => Ok(Sexp::MissingValue),
    }
}

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}
