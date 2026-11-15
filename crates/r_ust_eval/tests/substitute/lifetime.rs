use super::*;
use r_ust_sexp::{ClosureEnvironment, EvaluationOrigin, Force, PromiseState};

#[test]
fn substituted_call_runtime_roots_survive_origin_and_consumer_drop(
) -> Result<(), Box<dyn std::error::Error>> {
    let result = {
        let frame = Environment::global().child()?;
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        let closure = Sexp::Closure {
            attributes: Vec::new(),
            parameters: vec![],
            body: Box::new(Sexp::Symbol("marker".into())),
            environment: ClosureEnvironment::Captured(frame.clone()),
        };
        let promise = PromiseState::new(&closure, &frame, EvaluationOrigin::Global, true)?;
        let Force::Evaluate(lease) = promise.begin_force()? else {
            return Err("new promise".into());
        };
        lease.complete(&Sexp::Integer(vec![Some(99)]))?;
        frame.insert("f".into(), &Sexp::Promise { state: promise })?;
        frame.insert("e".into(), &Sexp::Environment(frame.clone()))?;
        let syntax = Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::named(
                "leaf",
                Sexp::Call {
                    function: Box::new(Sexp::Symbol("g".into())),
                    arguments: vec![CallArgument::positional(Sexp::Symbol("e".into()))],
                },
            )],
        };
        RuntimeState::default().eval(&call(syntax, Sexp::Environment(frame)))?
    };
    let alias = result.clone();
    drop(result);
    let Sexp::Call {
        function,
        arguments,
    } = &alias
    else {
        return Err("call".into());
    };
    assert_eq!(arguments[0].name.as_deref(), Some("leaf"));
    let Sexp::Call {
        arguments: nested, ..
    } = &arguments[0].value
    else {
        return Err("nested call".into());
    };
    let Sexp::Environment(frame) = &nested[0].value else {
        return Err("environment".into());
    };
    assert_eq!(
        frame.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(capture),
        ..
    } = function.as_ref()
    else {
        return Err("closure".into());
    };
    assert_eq!(capture, frame);
    let mut consumer = RuntimeState::default();
    assert_eq!(
        consumer.eval(&Sexp::Call {
            function: function.clone(),
            arguments: vec![]
        })?,
        Sexp::Integer(vec![Some(42)])
    );
    drop(consumer);
    assert_eq!(
        frame.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    Ok(())
}
