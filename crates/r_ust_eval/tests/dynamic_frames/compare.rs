use super::{
    double,
    node::{atom, Node},
};
use r_ust_sexp::{canonical_language, ClosureEnvironment, EnvironmentRole, ListElement, Sexp};

pub(super) fn compare(node: &Node, value: &Sexp) -> Result<(), String> {
    let valid = match value {
        Sexp::Nil => node.tag == "NULL",
        Sexp::MissingValue | Sexp::MissingArg(_) => node.tag == "missing",
        Sexp::Symbol(name) => node.tag == "symbol" && atom(&node.payload)? == *name,
        Sexp::Logical(values) => {
            node.tag == "logical"
                && node.names.is_none()
                && values
                    .iter()
                    .map(|value| match value {
                        Some(true) => "1",
                        Some(false) => "0",
                        None => "NA",
                    })
                    .eq(node.values())
        }
        Sexp::Integer(values) => {
            node.tag == "integer"
                && node.names.is_none()
                && values
                    .iter()
                    .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                    .eq(node.values())
        }
        Sexp::Real(values) => {
            if node.tag != "double" || node.names.is_some() || values.len() != node.size {
                return Err("double shape".into());
            }
            for (expected, actual) in node.values().into_iter().zip(values) {
                if expected == "NA" {
                    if !actual.is_nan() {
                        return Err("double missing value".into());
                    }
                } else if double::bits(expected)? != actual.to_bits() {
                    return Err("double bits".into());
                }
            }
            true
        }
        Sexp::String(values) => {
            if node.tag != "character" || node.names.is_some() || values.len() != node.size {
                return Err("character shape".into());
            }
            for (expected, actual) in node.values().into_iter().zip(values) {
                let expected = if expected == "NA" {
                    None
                } else {
                    Some(atom(expected)?)
                };
                if expected != *actual {
                    return Err("character value".into());
                }
            }
            true
        }
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
            elements(node, values, "list")?;
            true
        }
        Sexp::Vector(values) => {
            if node.tag != "list" || node.names.is_some() || node.size != values.len() {
                return Err("list shape".into());
            }
            for (expected, actual) in node.children.iter().zip(values) {
                compare(expected, actual)?;
            }
            true
        }
        Sexp::PairList(values) => {
            elements(node, values.as_slice(), "pairlist")?;
            true
        }
        Sexp::Call {
            function,
            arguments,
        } => {
            if node.tag != "language" || node.size != arguments.len() + 1 {
                return Err("call shape".into());
            }
            let names =
                std::iter::once(None).chain(arguments.iter().map(|arg| arg.name.as_deref()));
            names_match(node, names.collect())?;
            compare(&node.children[0], function)?;
            for (expected, argument) in node.children[1..].iter().zip(arguments) {
                compare(expected, &argument.value)?;
            }
            true
        }
        Sexp::Environment(environment) => {
            let role = match environment.role().map_err(|error| error.to_string())? {
                EnvironmentRole::Global => "TRUE,FALSE,FALSE,FALSE",
                EnvironmentRole::Base => "FALSE,TRUE,FALSE,FALSE",
                EnvironmentRole::BaseNamespace => "FALSE,FALSE,TRUE,FALSE",
                EnvironmentRole::Empty => "FALSE,FALSE,FALSE,TRUE",
                EnvironmentRole::Ordinary | EnvironmentRole::Autoloads => "FALSE,FALSE,FALSE,FALSE",
            };
            node.tag == "environment" && atom(&node.payload)? == role
        }
        Sexp::Closure {
            parameters,
            body,
            environment,
        } => {
            if node.tag != "closure" {
                return Err("closure type".into());
            }
            let fields = parameters
                .iter()
                .map(|parameter| {
                    Ok(ListElement {
                        name: Some(parameter.name.clone()),
                        value: match &parameter.default {
                            Some(value) => {
                                canonical_language(value).map_err(|error| error.to_string())?
                            }
                            None => Sexp::MissingValue,
                        },
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            compare(&node.children[0], &Sexp::pairlist(fields))?;
            compare(
                &node.children[1],
                &canonical_language(body).map_err(|error| error.to_string())?,
            )?;
            let ClosureEnvironment::Captured(frame) = environment else {
                return Err("unrooted source template returned".into());
            };
            compare(&node.children[2], &Sexp::Environment(frame.clone()))?;
            true
        }
        _ => return Err(format!("unsupported actual node: {value:?}")),
    };
    if !valid {
        return Err(format!(
            "typed mismatch: expected {node:?}, actual {value:?}"
        ));
    }
    Ok(())
}

fn elements(node: &Node, values: &[ListElement], tag: &str) -> Result<(), String> {
    if node.tag != tag || node.size != values.len() {
        return Err(format!("{tag} shape"));
    }
    names_match(
        node,
        values.iter().map(|value| value.name.as_deref()).collect(),
    )?;
    for (expected, actual) in node.children.iter().zip(values) {
        compare(expected, &actual.value)?;
    }
    Ok(())
}

fn names_match(node: &Node, names: Vec<Option<&str>>) -> Result<(), String> {
    let actual = names.iter().any(Option::is_some).then(|| {
        names
            .iter()
            .map(|name| name.unwrap_or("").to_string())
            .collect::<Vec<_>>()
    });
    if actual != node.names {
        return Err(format!("names mismatch: {:?} != {actual:?}", node.names));
    }
    Ok(())
}
