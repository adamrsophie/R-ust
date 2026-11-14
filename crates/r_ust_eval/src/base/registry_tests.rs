use std::collections::BTreeSet;

use r_ust_sexp::canonical_language;

use super::*;

#[test]
fn base_registry_enumeration_is_unique_complete() {
    let expected: BTreeSet<_> = definitions()
        .map(|(name, _)| *name)
        .chain(constants::iter().map(|(name, _)| name))
        .chain(
            crate::primitive_calls::names::NAMES
                .iter()
                .filter(|name| PrimitiveId::from_name(name).is_some())
                .copied(),
        )
        .collect();
    let mut names = BTreeSet::new();
    for (name, value) in iter().unwrap() {
        assert!(names.insert(name), "duplicate base binding: {name}");
        assert!(matches!(
            value,
            Sexp::Closure { .. }
                | Sexp::Primitive(_)
                | Sexp::Logical(_)
                | Sexp::Real(_)
                | Sexp::String(_)
        ));
    }
    assert_eq!(names, expected);
}

#[test]
fn base_registry_templates_preserve_formals_and_contain_no_session_roots() {
    for (name, source) in definitions() {
        let parsed = Parser.parse_all(source).unwrap();
        let [Sexp::Function { parameters, .. }] = parsed.expressions.as_slice() else {
            panic!("invalid source closure: {name}");
        };
        let Sexp::Closure {
            parameters: actual,
            body,
            environment,
        } = iter()
            .unwrap()
            .find(|(key, _)| key == name)
            .unwrap()
            .1
            .clone()
        else {
            panic!("missing closure: {name}");
        };
        assert_eq!(&actual, parameters, "{name}");
        assert_eq!(environment, ClosureEnvironment::Base, "{name}");
        for parameter in &actual {
            if let Some(default) = &parameter.default {
                assert!(canonical_language(default).is_ok(), "{name}");
            }
        }
        assert_root_free_body(&body);
    }
}

#[test]
fn base_registry_bootstrap_propagates_malformed_and_unresolved_errors() {
    for (source, expected) in [
        (
            "1L",
            EvalError::NotImplemented("base closure bootstrap shape"),
        ),
        (
            "function(x) x; function(y) y",
            EvalError::NotImplemented("base closure bootstrap shape"),
        ),
        (
            "function() .Internal(unknown_internal())",
            EvalError::NotImplemented("unresolved R internal"),
        ),
        (
            "function() .Internal()",
            EvalError::Arity("invalid base .Internal call".into()),
        ),
        (
            "function() .Internal(1L)",
            EvalError::Type("invalid base .Internal argument".into()),
        ),
        (
            "function() .Internal((function() 1L)())",
            EvalError::Type("invalid base internal identity".into()),
        ),
    ] {
        assert_eq!(template("invalid", source), Err(expected), "{source}");
    }
    let source = "function(";
    let error = Parser.parse_all(source).unwrap_err();
    assert_eq!(
        template("invalid", source),
        Err(EvalError::Parse(format!(
            "base bootstrap: {}",
            error.message
        )))
    );
}

fn assert_root_free_body(value: &Sexp) {
    match value {
        Sexp::Call {
            function,
            arguments,
        } => {
            assert_root_free_body(function);
            for argument in arguments {
                assert_root_free_body(&argument.value);
            }
        }
        Sexp::InternalCall { arguments, .. } => {
            for argument in arguments {
                assert_root_free_body(&argument.value);
            }
        }
        Sexp::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert_root_free_body(condition);
            assert_root_free_body(then_branch);
            if let Some(branch) = else_branch {
                assert_root_free_body(branch);
            }
        }
        Sexp::Block(values) => {
            for value in values {
                assert_root_free_body(value);
            }
        }
        Sexp::Assignment { value, .. } | Sexp::Unary { value, .. } | Sexp::Parenthesized(value) => {
            assert_root_free_body(value);
        }
        Sexp::Binary { left, right, .. } => {
            assert_root_free_body(left);
            assert_root_free_body(right);
        }
        Sexp::Nil
        | Sexp::Symbol(_)
        | Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_) => {}
        Sexp::MissingArg(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::Factor { .. }
        | Sexp::DataFrame { .. }
        | Sexp::Matrix { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
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
        | Sexp::MemberAssignment { .. }
        | Sexp::NamesAssignment { .. } => panic!("unexpected bootstrap value: {value:?}"),
    }
}
