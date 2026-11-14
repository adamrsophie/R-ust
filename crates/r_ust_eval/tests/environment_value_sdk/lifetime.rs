use super::*;

#[test]
fn environment_value_survives_origin_and_consumer_through_list_promise_and_closure() -> Result {
    let exported = {
        let mut origin = Frontend::default();
        let function = origin
            .eval_source("function() NULL")?
            .pop()
            .ok_or("closure")?;
        let Sexp::Closure {
            environment: ClosureEnvironment::Captured(global),
            ..
        } = function
        else {
            panic!("captured global")
        };
        let frame = global.child()?;
        frame.insert("x".into(), &Sexp::Integer(vec![Some(1)]))?;
        let value = Sexp::Environment(frame.clone());
        let pending = PromiseState::new(&value, &frame, EvaluationOrigin::Global, true)?;
        let cached = PromiseState::new(&value, &frame, EvaluationOrigin::Global, true)?;
        let Force::Evaluate(lease) = cached.begin_force()? else {
            panic!("pending")
        };
        lease.complete(&value)?;
        let closure = Sexp::Closure {
            parameters: vec![],
            body: Box::new(value.clone()),
            environment: ClosureEnvironment::Captured(frame.clone()),
        };
        let bundle = Sexp::Vector(vec![
            list(None, value),
            Sexp::Promise { state: pending },
            Sexp::Promise { state: cached },
            closure,
        ]);
        frame.insert("cycle".into(), &bundle)?;
        bind(origin.runtime_mut(), "bundle", bundle)?;
        origin.runtime().get("bundle")?.ok_or("bundle missing")?
    };
    let Sexp::Vector(values) = &exported else {
        panic!("vector")
    };
    let Sexp::List(elements) = &values[0] else {
        panic!("list")
    };
    let Sexp::Environment(frame) = &elements[0].value else {
        panic!("environment")
    };
    let alias = frame.clone();
    let mut consumer = Frontend::default();
    consumer.eval_source("x<-99L")?;
    let mut results = vec![
        consumer.runtime_mut().eval(&values[1])?,
        consumer.runtime_mut().eval(&values[2])?,
    ];
    results.push(consumer.runtime_mut().eval(&Sexp::Call {
        function: Box::new(values[3].clone()),
        arguments: vec![],
    })?);
    drop(exported);
    alias.insert("x".into(), &Sexp::Integer(vec![Some(42)]))?;
    assert_eq!(
        consumer.runtime().get("x")?,
        Some(Sexp::Integer(vec![Some(99)]))
    );
    drop(consumer);
    for value in results {
        let Sexp::Environment(frame) = value else {
            panic!("returned environment")
        };
        assert_eq!(frame, alias);
        assert_eq!(frame.get_local("x")?, Some(Sexp::Integer(vec![Some(42)])));
    }
    Ok(())
}
