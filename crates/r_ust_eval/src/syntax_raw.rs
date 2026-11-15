mod assignments;

use crate::{CallArgument, ListElement, Sexp};

pub(super) fn call(value: &Sexp) -> Option<Sexp> {
    let (name, values) = match value {
        Sexp::Function { parameters, body } => (
            "function",
            vec![
                Sexp::pairlist(
                    parameters
                        .iter()
                        .map(|parameter| ListElement {
                            name: Some(parameter.name.clone()),
                            value: parameter.default.clone().unwrap_or(Sexp::MissingValue),
                        })
                        .collect(),
                ),
                body.as_ref().clone(),
                Sexp::Nil,
            ],
        ),
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut values = vec![condition.as_ref().clone(), then_branch.as_ref().clone()];
            values.extend(else_branch.as_deref().cloned());
            ("if", values)
        }
        Sexp::For {
            variable,
            sequence,
            body,
        } => (
            "for",
            vec![
                Sexp::Symbol(variable.clone()),
                sequence.as_ref().clone(),
                body.as_ref().clone(),
            ],
        ),
        Sexp::While { condition, body } => (
            "while",
            vec![condition.as_ref().clone(), body.as_ref().clone()],
        ),
        Sexp::Repeat { body } => ("repeat", vec![body.as_ref().clone()]),
        Sexp::Break => ("break", vec![]),
        Sexp::Next => ("next", vec![]),
        Sexp::Block(values) => ("{", values.clone()),
        Sexp::Subset { value, index } => (
            "[",
            vec![value.as_ref().clone(), optional(index.as_deref())],
        ),
        Sexp::Subset2D { value, row, col } => (
            "[",
            vec![
                value.as_ref().clone(),
                optional(row.as_deref()),
                optional(col.as_deref()),
            ],
        ),
        Sexp::ExtractArguments { value, arguments } => {
            let mut values = vec![CallArgument::positional(value.as_ref().clone())];
            values.extend_from_slice(arguments);
            return Some(Sexp::Call {
                function: Box::new(Sexp::Symbol("[[".into())),
                arguments: values,
            });
        }
        Sexp::Extract { value, index } => {
            ("[[", vec![value.as_ref().clone(), index.as_ref().clone()])
        }
        Sexp::Dollar { value, name } => ("$", vec![value.as_ref().clone(), name.to_sexp()]),
        _ => return assignments::call(value),
    };
    Some(named(name, values))
}

fn named(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}

fn optional(value: Option<&Sexp>) -> Sexp {
    value.cloned().unwrap_or(Sexp::MissingValue)
}
