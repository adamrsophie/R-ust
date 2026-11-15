use super::*;
use crate::RuntimeState;
use r_ust_sexp::{ClosureEnvironment, EnvironmentRole, ListElement, ListObject, PairListObject};

pub(super) fn install_inventory(frontend: &mut Frontend, binding: &str, replacement: &str) {
    let namespace = frontend
        .runtime
        .global
        .session_root(EnvironmentRole::BaseNamespace)
        .unwrap();
    let parsed = r_ust_parse::Parser.parse_all(replacement).unwrap();
    let [Sexp::Function { parameters, body }] = parsed.expressions.as_slice() else {
        panic!("replacement closure");
    };
    let replacement = Sexp::Closure {
        parameters: parameters.clone(),
        body: body.clone(),
        environment: ClosureEnvironment::Captured(frontend.runtime.global.clone()),
        attributes: Vec::new(),
    };
    let mut bindings = crate::base::iter()
        .unwrap()
        .map(|(name, template)| {
            let mut value = if name == binding {
                replacement.clone()
            } else {
                template.clone()
            };
            if let Sexp::Closure { environment, .. } = &mut value {
                if matches!(environment, ClosureEnvironment::Base) {
                    *environment = ClosureEnvironment::Captured(namespace.clone());
                }
            }
            (name.into(), value)
        })
        .collect::<Vec<_>>();
    for (name, role) in [
        (".GlobalEnv", EnvironmentRole::Global),
        (".AutoloadEnv", EnvironmentRole::Autoloads),
        (".BaseNamespaceEnv", EnvironmentRole::BaseNamespace),
    ] {
        bindings.push((
            name.into(),
            Sexp::Environment(frontend.runtime.global.session_root(role).unwrap()),
        ));
    }
    frontend.runtime.global.install_base(&bindings).unwrap();
}

#[test]
fn abort_restart_has_an_explicit_unsupported_guard_and_cleans_the_session() {
    let mut frontend = Frontend::default();
    let result = frontend.eval_source("f<-function(){on.exit(cat('ABORT_CLEANUP|'));invokeRestart('abort');cat('UNREACHABLE|')};withRestarts(f(),r=function()0L)");
    assert!(matches!(
        result,
        Err(EvalError::NotImplemented(
            "top-level abort restart transfer"
        ))
    ));
    assert_eq!(
        streams(&mut frontend).0,
        include_str!("raw/restart_abort_open.stdout")
    );
    assert!(frontend.runtime.conditions.restarts.is_none());
    assert!(frontend.runtime.conditions.head.is_none());
    assert!(frontend.runtime.control.contexts.is_empty());
}

#[test]
fn restart_stack_retains_runtime_values_and_restores_the_existing_root() {
    let mut runtime = RuntimeState::default();
    let retained = runtime.global.child().unwrap();
    retained
        .insert("payload".into(), &Sexp::Integer(vec![Some(41)]))
        .unwrap();
    let restart = Sexp::List(vec![
        ListElement {
            name: None,
            value: Sexp::String(vec![Some("outer".into())]),
        },
        ListElement {
            name: None,
            value: Sexp::Environment(retained.clone()),
        },
    ]);
    let object = ListObject::from_value(&runtime.global, &restart).unwrap();
    let head = PairListObject::cons(
        &runtime.global,
        &ListElement {
            name: None,
            value: Sexp::ListObject(object),
        },
        None,
    )
    .unwrap();
    runtime.conditions.restarts = Some(head.clone());
    drop(restart);
    drop(retained);
    let expressions = r_ust_parse::Parser
        .parse_all("withRestarts(7L,inner=function()NULL)")
        .unwrap()
        .expressions;
    assert_eq!(
        runtime.eval(&expressions[0]).unwrap(),
        Sexp::Integer(vec![Some(7)])
    );
    assert!(runtime
        .conditions
        .restarts
        .as_ref()
        .unwrap()
        .same_object(&head));
    let Sexp::ListObject(value) = head.car().unwrap().value else {
        panic!("rooted restart");
    };
    let Sexp::Environment(exit) = value.element(1).unwrap() else {
        panic!("retained exit");
    };
    assert_eq!(
        exit.get_local("payload").unwrap(),
        Some(Sexp::Integer(vec![Some(41)]))
    );
}

#[test]
fn retained_restart_cannot_target_a_later_or_other_runtime_activation() {
    let mut first = Frontend::default();
    let restart = first
        .eval_source("withRestarts(findRestart('r'),r=function()9L)")
        .unwrap()
        .pop()
        .unwrap();
    let mut second = Frontend::default();
    second
        .runtime
        .global
        .insert("old".into(), &restart)
        .unwrap();
    drop(first);
    let values = second.eval_source("withRestarts(tryCatch(invokeRestart(old),error=function(e)conditionMessage(e)),r=function()99L)").unwrap();
    assert_eq!(
        values.last(),
        Some(&Sexp::String(vec![Some("restart not on stack".into())]))
    );
    assert!(second.runtime.conditions.restarts.is_none());
    assert!(second.runtime.control.contexts.is_empty());
}
