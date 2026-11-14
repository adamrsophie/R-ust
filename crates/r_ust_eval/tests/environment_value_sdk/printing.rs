use super::*;

fn normalize(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<environment: 0x") {
        let token = start + "<environment: 0x".len();
        let end = rest[token..].find('>').expect("terminated marker") + token;
        assert!(rest[token..end]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit()));
        assert!(end > token);
        out.push_str(&rest[..start]);
        out.push_str("<environment: 0xTOKEN>");
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

#[test]
fn autoloads_print_attributes() -> Result {
    let global = Environment::global();
    let autoloads = global.session_root(EnvironmentRole::Autoloads)?;
    let value = Sexp::Environment(autoloads.clone());
    let nested = Sexp::List(vec![
        ListElement {
            name: Some("a".into()),
            value: list(Some("b"), value.clone()),
        },
        ListElement {
            name: Some("z".into()),
            value: value.clone(),
        },
    ]);
    for (value, expected) in [
        (
            value.clone(),
            include_str!("../data/environment-values/autoloads.txt"),
        ),
        (
            list(None, value.clone()),
            include_str!("../data/environment-values/autoloads-list.txt"),
        ),
        (
            nested,
            include_str!("../data/environment-values/autoloads-nested.txt"),
        ),
    ] {
        assert_eq!(normalize(&(format_sexp(&value)? + "\n")), expected);
    }
    assert_eq!(format_sexp(&value)?, format_sexp(&value.clone())?);
    assert_eq!(autoloads.local_len()?, 1);
    assert_eq!(autoloads.name_attribute()?, Some("Autoloads"));
    assert_ne!(
        normalize("<environment: 0x12>\nattr(,\"wrong\")"),
        normalize("<environment: 0x45>\nattr(,\"name\")")
    );
    assert_ne!(
        normalize(" <environment: 0x12>"),
        normalize("<environment: 0x45>")
    );
    for (role, expected) in [
        (
            EnvironmentRole::Global,
            include_str!("../data/environment-values/global.txt"),
        ),
        (
            EnvironmentRole::Base,
            include_str!("../data/environment-values/base.txt"),
        ),
        (
            EnvironmentRole::BaseNamespace,
            include_str!("../data/environment-values/namespace.txt"),
        ),
        (
            EnvironmentRole::Empty,
            include_str!("../data/environment-values/empty.txt"),
        ),
    ] {
        assert_eq!(
            format_sexp(&Sexp::Environment(global.session_root(role)?))? + "\n",
            expected
        );
    }
    Ok(())
}

#[test]
fn existing_list_layout_matches_independent_r() -> Result {
    for (source, expected) in [
        (
            "list(1L,2L)",
            include_str!("../data/environment-values/plain-list.txt"),
        ),
        (
            "list(a=list(b=1L),z=2L)",
            include_str!("../data/environment-values/nested-list.txt"),
        ),
        (
            "list(list(),3L)",
            include_str!("../data/environment-values/empty-list.txt"),
        ),
    ] {
        let value = Frontend::default()
            .eval_source(source)?
            .pop()
            .ok_or("value")?;
        assert_eq!(format_sexp(&value)? + "\n", expected);
    }
    Ok(())
}

#[test]
fn r128_environment_print_rows_match_full_print_and_effect_bytes() -> Result {
    let root = Environment::global();
    let autoloads = Sexp::Environment(root.session_root(EnvironmentRole::Autoloads)?);
    let roots = [
        EnvironmentRole::Global,
        EnvironmentRole::Base,
        EnvironmentRole::BaseNamespace,
        EnvironmentRole::Empty,
    ]
    .into_iter()
    .map(|role| {
        Ok(ListElement {
            name: None,
            value: Sexp::Environment(root.session_root(role)?),
        })
    })
    .collect::<Result<Vec<_>>>()?;
    for (value, expected) in [
        (
            autoloads.clone(),
            include_str!("../data/environment-values/autoload_print.txt"),
        ),
        (
            list(None, autoloads.clone()),
            include_str!("../data/environment-values/autoload_list_print.txt"),
        ),
        (
            list(None, list(None, autoloads.clone())),
            include_str!("../data/environment-values/autoload_nested_print.txt"),
        ),
        (
            list(Some("a"), autoloads),
            include_str!("../data/environment-values/autoload_named_print.txt"),
        ),
        (
            Sexp::List(roots),
            include_str!("../data/environment-values/roots_nested_print.txt"),
        ),
    ] {
        assert_eq!(normalize(&format_sexp(&value)?), expected);
        let mut runtime = RuntimeState::default();
        let returned = call(&mut runtime, "print", vec![value.clone()])?;
        assert_eq!(returned, value);
        let events = runtime.take_output();
        let [r_ust_eval::OutputEvent::Stdout(text)] = events.as_slice() else {
            panic!("one print event")
        };
        assert_eq!(normalize(text), expected.to_string() + "\n");
        assert!(!runtime.is_visible());
    }
    Ok(())
}
