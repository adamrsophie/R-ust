use std::sync::OnceLock;

use r_ust_sexp::ClosureEnvironment;

use super::{EvalError, Parser, Sexp};

// R 4.6.0 src/library/base/R/stop.R: Copyright (C) 1995-2025 The R Core Team
// R 4.6.0 src/library/base/R/message.R: Copyright (C) 1995-2021 The R Core Team
// R 4.6.0 src/library/base/R/New-Internal.R: Copyright (C) 1995-2023 The R Core Team
// R 4.6.0 src/library/base/R/vector.R: Copyright (C) 1995-2013 The R Core Team
// Part of the R package, https://www.R-project.org
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// A copy of the GNU General Public License is available at
// https://www.R-project.org/Licenses/
const DEFINITIONS: &[(&str, &str)] = &[
    (
        "vector",
        "function(mode = \"logical\", length = 0L) .Internal(vector(mode, length))",
    ),
    (
        "logical",
        "function(length = 0L) .Internal(vector(\"logical\", length))",
    ),
    (
        "integer",
        "function(length = 0L) .Internal(vector(\"integer\", length))",
    ),
    (
        "double",
        "function(length = 0L) .Internal(vector(\"double\", length))",
    ),
    (
        "numeric",
        "function(length = 0L) .Internal(vector(\"double\", length))",
    ),
    (
        "character",
        "function(length = 0L) .Internal(vector(\"character\", length))",
    ),
    (
        "stop",
        r#"function(..., call. = TRUE, domain = NULL)
{
    if (...length() == 1L && inherits(..1, "condition")) {
        cond <- ..1
        if(nargs() > 1L)
            warning("additional arguments ignored in stop()")
        message <- conditionMessage(cond)
        call <- conditionCall(cond)
        .Internal(.signalCondition(cond, message, call))
        .Internal(.dfltStop(message, call))
    } else
        .Internal(stop(call., .makeMessage(..., domain = domain)))
}"#,
    ),
    (
        ".makeMessage",
        r#"function(..., domain = NULL, appendLF = FALSE)
 {
    msg <- if(...length()) {
        args <- lapply(list(...), as.character)
        if(is.null(domain) || !is.na(domain))
            args <- .Internal(gettext(domain, unlist(args), TRUE))
        paste(args, collapse = "")
    } else ""
    if(appendLF) paste0(msg, "\n") else msg
}"#,
    ),
    (
        "inherits",
        "function(x, what, which = FALSE) .Internal(inherits(x, what, which))",
    ),
];

pub(super) fn lookup(name: &str) -> Result<Option<Sexp>, EvalError> {
    static BASE: OnceLock<Result<Vec<(String, Sexp)>, EvalError>> = OnceLock::new();
    if !DEFINITIONS.iter().any(|(key, _)| *key == name) {
        return Ok(None);
    }
    let registry = BASE.get_or_init(|| {
        DEFINITIONS
            .iter()
            .map(|(name, source)| {
                let parsed = Parser.parse_all(source).map_err(|error| {
                    EvalError::Parse(format!("base bootstrap: {}", error.message))
                })?;
                let [Sexp::Function { parameters, body }] = parsed.expressions.as_slice() else {
                    return Err(EvalError::NotImplemented("base closure bootstrap shape"));
                };
                let mut body = body.clone();
                lower(&mut body)?;
                Ok((
                    (*name).into(),
                    Sexp::Closure {
                        parameters: parameters.clone(),
                        body,
                        environment: ClosureEnvironment::Base,
                    },
                ))
            })
            .collect()
    });
    match registry {
        Ok(values) => Ok(values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())),
        Err(error) => Err(error.clone()),
    }
}

fn lower(value: &mut Sexp) -> Result<(), EvalError> {
    match value {
        Sexp::Call {
            function,
            arguments,
        } => {
            if matches!(function.as_ref(), Sexp::Symbol(name) if name == ".Internal") {
                let [argument] = arguments.as_mut_slice() else {
                    return Err(EvalError::Arity("invalid base .Internal call".into()));
                };
                let Sexp::Call {
                    function,
                    arguments,
                } = &mut argument.value
                else {
                    return Err(EvalError::Type("invalid base .Internal argument".into()));
                };
                let Sexp::Symbol(name) = function.as_ref() else {
                    return Err(EvalError::Type("invalid base internal identity".into()));
                };
                let operation = super::internal::resolve(name)
                    .ok_or(EvalError::NotImplemented("unresolved R internal"))?;
                for argument in arguments.iter_mut() {
                    lower(&mut argument.value)?;
                }
                *value = Sexp::InternalCall {
                    operation,
                    arguments: std::mem::take(arguments),
                };
            } else {
                lower(function)?;
                for argument in arguments {
                    lower(&mut argument.value)?;
                }
            }
        }
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            lower(condition)?;
            lower(then_branch)?;
            if let Some(branch) = else_branch {
                lower(branch)?;
            }
        }
        Sexp::Block(values) => {
            for value in values {
                lower(value)?;
            }
        }
        Sexp::Assignment { value, .. } | Sexp::Unary { value, .. } | Sexp::Parenthesized(value) => {
            lower(value)?
        }
        Sexp::Binary { left, right, .. } => {
            lower(left)?;
            lower(right)?;
        }
        Sexp::Nil | Sexp::Symbol(_) | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::String(_) => {}
        Sexp::MissingArg(_)
        | Sexp::Real(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::InternalCall { .. }
        | Sexp::Function { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. } => {
            return Err(EvalError::NotImplemented("base bootstrap expression"))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_internal_rejects_unresolved_identities() {
        let mut expression = Parser
            .parse_all(".Internal(unknown_internal())")
            .unwrap()
            .expressions
            .remove(0);
        assert_eq!(
            lower(&mut expression),
            Err(EvalError::NotImplemented("unresolved R internal"))
        );
    }
}
