use super::*;
use crate::{
    canonical_actual_call, CallArgument, ClosureEnvironment, EvaluationOrigin, Force, Sexp,
};

fn closure(frame: &super::super::Environment) -> Sexp {
    Sexp::Closure {
        attributes: Vec::new(),
        parameters: vec![],
        body: Box::new(Sexp::Symbol("marker".into())),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn substitute_prexpr_and_quote_carriers_release_last_runtime_root(
) -> Result<(), Box<dyn std::error::Error>> {
    for cached in [false, true] {
        let heap = Arc::new(Heap::default());
        let owner = heap.session();
        let frame = heap.session();
        let original = Sexp::Call {
            function: Box::new(closure(&frame)),
            arguments: vec![CallArgument::named(
                "frame",
                Sexp::Environment(frame.clone()),
            )],
        };
        let promise = crate::PromiseState::new(&original, &frame, EvaluationOrigin::Global, true)?;
        let ids = [frame.0.id(), promise.0.id()];
        frame.insert(
            "promise".into(),
            &Sexp::Promise {
                state: promise.clone(),
            },
        )?;
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        if cached {
            let Force::Evaluate(lease) = promise.begin_force()? else {
                return Err("new promise".into());
            };
            lease.complete(&Sexp::Integer(vec![Some(99)]))?;
        }
        let extracted = canonical_actual_call(&promise.expression()?)?;
        assert_eq!(extracted, original);
        owner.insert("result".into(), &extracted)?;
        drop(extracted);
        drop(original);
        drop(promise);
        drop(frame);
        let result = owner.get_local("result")?.ok_or("result")?;
        drop(owner);
        let alias = result.clone();
        drop(result);
        {
            let Sexp::Call {
                function,
                arguments,
            } = &alias
            else {
                return Err("call".into());
            };
            let Sexp::Environment(frame) = &arguments[0].value else {
                return Err("frame".into());
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
            assert_eq!(frame, capture);
        }
        for id in ids {
            assert!(heap.lock().node(id).is_ok());
        }
        drop(alias);
        for id in ids {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    }
    Ok(())
}

#[test]
fn quote_runtime_leaf_foreign_import_is_transactional() -> Result<(), Box<dyn std::error::Error>> {
    let heap = Arc::new(Heap::default());
    let foreign = Arc::new(Heap::default());
    let holder = heap.session();
    let frame = foreign.session();
    let id = frame.0.id();
    holder.insert("result".into(), &Sexp::Integer(vec![Some(7)]))?;
    let raw = Sexp::Call {
        function: Box::new(closure(&frame)),
        arguments: vec![CallArgument::positional(Sexp::Environment(frame.clone()))],
    };
    let quoted = canonical_actual_call(&raw)?;
    assert_eq!(
        holder.insert("result".into(), &quoted),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(
        holder.get_local("result")?,
        Some(Sexp::Integer(vec![Some(7)]))
    );
    drop(raw);
    drop(quoted);
    drop(frame);
    assert!(matches!(
        foreign.lock().node(id),
        Err(GraphError::StaleNode)
    ));
    Ok(())
}
