use super::*;

#[test]
fn r128_root_metadata_matches_and_base_count_is_supported_inventory() -> Result {
    let root = Environment::global();
    let mut runtime = RuntimeState::default();
    for row in include_str!("../data/environment-values/metadata.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [id, kind, class, mode, length] = fields.as_slice() else {
            panic!("five fields")
        };
        let role = match *id {
            "global_value" => EnvironmentRole::Global,
            "base_value" => EnvironmentRole::Base,
            "empty_value" => EnvironmentRole::Empty,
            "namespace_value" => EnvironmentRole::BaseNamespace,
            "autoload_value" => EnvironmentRole::Autoloads,
            "ordinary_value" => EnvironmentRole::Ordinary,
            other => panic!("unknown reference row {other}"),
        };
        let frame = if role == EnvironmentRole::Ordinary {
            root.child()?
        } else {
            root.session_root(role)?
        };
        let value = Sexp::Environment(frame);
        for (name, expected) in [("typeof", kind), ("class", class), ("mode", mode)] {
            assert_eq!(
                call(&mut runtime, name, vec![value.clone()])?,
                Sexp::String(vec![Some(expected.to_string())]),
                "{id}/{name}"
            );
        }
        if matches!(role, EnvironmentRole::Base | EnvironmentRole::BaseNamespace) {
            assert_eq!(
                *length, "1407",
                "reference full-R inventory remains explicit"
            );
        } else {
            assert_eq!(
                call(&mut runtime, "length", vec![value])?,
                Sexp::Integer(vec![Some(length.parse()?)]),
                "{id}"
            );
        }
    }
    let inventory = [
        ("one".into(), Sexp::Nil),
        ("two".into(), Sexp::Integer(vec![Some(2)])),
    ];
    root.install_base(&inventory)?;
    for role in [EnvironmentRole::Base, EnvironmentRole::BaseNamespace] {
        assert_eq!(
            call(
                &mut runtime,
                "length",
                vec![Sexp::Environment(root.session_root(role)?)]
            )?,
            Sexp::Integer(vec![Some(2)])
        );
    }
    Ok(())
}

#[test]
fn environment_value_length_counts_local_bindings_without_forcing() -> Result {
    let root = Environment::global();
    root.insert("inherited".into(), &Sexp::Integer(vec![Some(9)]))?;
    let frame = root.child()?;
    let mut runtime = RuntimeState::default();
    let value = Sexp::Environment(frame.clone());
    assert_eq!(
        call(&mut runtime, "length", vec![value.clone()])?,
        Sexp::Integer(vec![Some(0)])
    );
    frame.insert("null".into(), &Sexp::Nil)?;
    assert_eq!(
        call(&mut runtime, "length", vec![value.clone()])?,
        Sexp::Integer(vec![Some(1)])
    );
    let pending = PromiseState::new(
        &Sexp::Symbol("must_not_force".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    frame.insert(
        "lazy".into(),
        &Sexp::Promise {
            state: pending.clone(),
        },
    )?;
    assert_eq!(
        call(&mut runtime, "length", vec![value.clone()])?,
        Sexp::Integer(vec![Some(2)])
    );
    frame.insert("null".into(), &Sexp::Integer(vec![Some(99)]))?;
    assert_eq!(
        call(&mut runtime, "length", vec![value])?,
        Sexp::Integer(vec![Some(2)])
    );
    assert!(matches!(pending.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
