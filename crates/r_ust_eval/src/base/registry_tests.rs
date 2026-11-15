#[path = "registry_tree.rs"]
mod tree;
use tree::assert_root_free_body;

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
                | Sexp::Bytecode(_)
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
            ..
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

#[test]
fn source_internal_bodies_remain_inspectable_and_execute() {
    for name in ["body", "formals", "bitwNot", "parent.env"] {
        let (_, source) = definitions().find(|(key, _)| *key == name).unwrap();
        let parsed = Parser.parse_all(source).unwrap();
        let [Sexp::Function { body: expected, .. }] = parsed.expressions.as_slice() else {
            panic!("expected source function");
        };
        let Sexp::Closure { body, .. } = template(name, source).unwrap() else {
            panic!("expected closure");
        };
        assert_eq!(
            canonical_language(&body),
            canonical_language(expected),
            "{name}"
        );
        assert!(canonical_language(&body).is_ok());
    }
    let mut frontend = crate::Frontend::default();
    assert_eq!(
        frontend.eval_source("bitwNot(1L)").unwrap(),
        [Sexp::Integer(vec![Some(-2)])]
    );
}
