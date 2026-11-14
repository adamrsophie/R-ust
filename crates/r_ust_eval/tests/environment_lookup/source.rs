use super::*;

pub(super) fn source_session() {
    let (identity, retained, pending) = {
        let mut source = Frontend::default();
        source.eval_source("sentinel <- 11L").unwrap();
        let identity = source.eval_source("identity").unwrap().pop().unwrap();
        let Sexp::Closure {
            environment: ClosureEnvironment::Captured(namespace),
            ..
        } = &identity
        else {
            panic!("base template was not materialized")
        };
        let retained = closure(namespace, Sexp::Symbol("sentinel".into()));
        let pending = r_ust_sexp::PromiseState::new(
            &Sexp::Symbol("sentinel".into()),
            namespace,
            r_ust_sexp::EvaluationOrigin::default(),
            true,
        )
        .unwrap();
        (identity, retained, pending)
    };
    let mut consumer = RuntimeState::default();
    bind(&mut consumer, "sentinel", integer(99));
    assert_eq!(
        consumer
            .eval(&Sexp::Call {
                function: Box::new(identity.clone()),
                arguments: vec![CallArgument::positional(integer(7))]
            })
            .unwrap(),
        integer(7)
    );
    assert_eq!(invoke(&mut consumer, &retained).unwrap(), integer(11));
    assert_eq!(
        consumer.eval(&Sexp::Promise { state: pending }).unwrap(),
        integer(11)
    );
    let legacy = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Symbol("sentinel".into())),
        environment: ClosureEnvironment::Base,
    };
    assert_eq!(invoke(&mut consumer, &legacy).unwrap(), integer(99));
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(namespace),
        ..
    } = identity
    else {
        panic!("missing capture")
    };
    assert_ne!(namespace, root(&consumer, EnvironmentRole::BaseNamespace));
    assert_eq!(
        namespace.get_local(".GlobalEnv").unwrap(),
        Some(Sexp::Environment(
            namespace.session_root(EnvironmentRole::Global).unwrap()
        ))
    );
}

pub(super) fn retained_store() {
    let (base, namespace, identity, count) = {
        let mut source = Frontend::default();
        source.eval_source("sentinel <- 11L").unwrap();
        let base = root(source.runtime(), EnvironmentRole::Base);
        let namespace = root(source.runtime(), EnvironmentRole::BaseNamespace);
        let identity = source.runtime().get("identity").unwrap().unwrap();
        let count = base.local_len().unwrap();
        (
            Sexp::Environment(base),
            Sexp::Environment(namespace),
            identity,
            count,
        )
    };
    let Sexp::Environment(base) = base else {
        panic!("missing base export")
    };
    let Sexp::Environment(namespace) = namespace else {
        panic!("missing namespace export")
    };
    let mut consumer = RuntimeState::default();
    bind(&mut consumer, "sentinel", integer(99));
    assert_ne!(base, namespace);
    assert!(count > 3);
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(capture),
        ..
    } = &identity
    else {
        panic!("missing capture")
    };
    assert_eq!(capture, &namespace);
    for frame in [&base, &namespace] {
        assert_eq!(frame.local_len().unwrap(), count);
        assert_eq!(frame.get_local("identity").unwrap(), Some(identity.clone()));
        assert_eq!(
            frame.insert("identity".into(), &integer(0)),
            Err(GraphError::BindingLocked)
        );
        assert_eq!(
            frame.insert("fresh_after_drop".into(), &integer(0)),
            Err(GraphError::EnvironmentLocked)
        );
        assert_eq!(frame.local_len().unwrap(), count);
        assert_eq!(frame.get_local("identity").unwrap(), Some(identity.clone()));
        assert_eq!(frame.get_local("fresh_after_drop").unwrap(), None);
    }
    assert_eq!(
        invoke(
            &mut consumer,
            &closure(&namespace, Sexp::Symbol("sentinel".into()))
        )
        .unwrap(),
        integer(11)
    );
    assert_eq!(consumer.get("sentinel").unwrap(), Some(integer(99)));
    assert_eq!(
        namespace.get_local(".AutoloadEnv").unwrap(),
        Some(Sexp::Environment(
            namespace.session_root(EnvironmentRole::Autoloads).unwrap()
        ))
    );
}
