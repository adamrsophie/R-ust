use super::*;
use crate::RuntimeState;

#[test]
fn installed_base_inventory_matches_root_free_templates() {
    let runtime = RuntimeState::default();
    let namespace = runtime
        .global
        .session_root(EnvironmentRole::BaseNamespace)
        .unwrap();
    let base = runtime.global.session_root(EnvironmentRole::Base).unwrap();
    assert!(!base.base_initialized().unwrap());
    initialize(&runtime.global).unwrap();
    let templates: Vec<_> = super::super::iter().unwrap().collect();
    assert_eq!(base.local_len().unwrap(), templates.len() + 3);
    assert_eq!(namespace.local_len().unwrap(), base.local_len().unwrap());
    assert_eq!(runtime.global.local_len().unwrap(), 0);
    for (name, template) in templates {
        let mut expected = template.clone();
        if let Sexp::Closure { environment, .. } = &mut expected {
            assert_eq!(environment, &ClosureEnvironment::Base);
            *environment = ClosureEnvironment::Captured(namespace.clone());
        }
        assert_eq!(
            base.get_local(name).unwrap(),
            Some(expected.clone()),
            "{name}"
        );
        assert_eq!(
            namespace.get_local(name).unwrap(),
            Some(expected.clone()),
            "{name}"
        );
        assert_eq!(runtime.get(name).unwrap(), Some(expected), "{name}");
    }
    for (name, role) in [
        (".GlobalEnv", EnvironmentRole::Global),
        (".AutoloadEnv", EnvironmentRole::Autoloads),
        (".BaseNamespaceEnv", EnvironmentRole::BaseNamespace),
    ] {
        assert_eq!(
            runtime.get(name).unwrap(),
            Some(Sexp::Environment(
                runtime.global.session_root(role).unwrap()
            ))
        );
    }
    for name in crate::primitive_calls::names::NAMES {
        if r_ust_sexp::PrimitiveId::from_name(name).is_none()
            && !super::super::iter().unwrap().any(|(key, _)| key == *name)
        {
            assert_eq!(runtime.get(name).unwrap(), None, "{name}");
        }
    }
    assert_eq!(runtime.get("not_a_registered_base_binding").unwrap(), None);
    let before = base.get_local("identity").unwrap();
    initialize(&namespace).unwrap();
    assert_eq!(base.get_local("identity").unwrap(), before);
}

#[test]
fn installed_sdk_inventory_is_preserved_even_when_empty() {
    for entries in [vec![], vec![("sdk".into(), Sexp::Integer(vec![Some(19)]))]] {
        let global = Environment::global();
        global.install_base(&entries).unwrap();
        initialize(&global).unwrap();
        let base = global.session_root(EnvironmentRole::Base).unwrap();
        assert_eq!(base.local_len().unwrap(), entries.len());
        assert_eq!(base.get_local("identity").unwrap(), None);
        for (name, value) in entries {
            assert_eq!(base.get_local(&name).unwrap(), Some(value));
        }
    }
}
