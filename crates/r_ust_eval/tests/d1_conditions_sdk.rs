use r_ust_eval::{EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{AssignmentOperator, CallArgument, InternalOperation, PrimitiveId, Sexp};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}

fn call(function: Sexp, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(function),
        arguments: values.into_iter().map(CallArgument::positional).collect(),
    }
}

#[test]
fn exported_calling_head_and_callback_outlive_origin_and_consumer() -> Result {
    let saved = {
        let mut origin = Frontend::default();
        origin.eval_source("x<-11L;withCallingHandlers(.Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE)),error=function(e){cat(x);x})")?
            .pop().ok_or("saved head")?
    };
    let Sexp::PairListObject(head) = &saved else {
        panic!("ordinary pairlist head")
    };
    let Sexp::ListObject(entry) = head.car()?.value else {
        panic!("ordinary entry")
    };
    assert_eq!(entry.element(1)?, Sexp::Nil);
    assert_eq!(entry.element(3)?, Sexp::Nil);
    let callback = entry.element(2)?;
    drop(entry);
    {
        let mut consumer = Frontend::default();
        consumer.eval_source("x<-99L")?;
        bind(consumer.runtime_mut(), "saved", saved.clone())?;
        let installed = consumer.eval_source(".Internal(.resetCondHands(saved));.Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE))")?.pop().ok_or("installed head")?;
        let Sexp::PairListObject(installed) = installed else {
            panic!("installed ordinary head")
        };
        assert!(installed.same_object(head));
        drop(installed);
        assert_eq!(
            consumer
                .eval_source(
                    "signalCondition(simpleError('C'));.Internal(.resetCondHands(NULL));x"
                )?
                .pop(),
            Some(Sexp::Integer(vec![Some(99)]))
        );
        assert_eq!(
            consumer.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("11".into())]
        );
        assert_eq!(
            consumer
                .runtime_mut()
                .eval(&call(callback.clone(), vec![Sexp::Nil]))?,
            Sexp::Integer(vec![Some(11)])
        );
        assert_eq!(
            consumer.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("11".into())]
        );
    }
    drop(saved);
    let mut final_runtime = RuntimeState::default();
    assert_eq!(
        final_runtime.eval(&call(callback, vec![Sexp::Nil]))?,
        Sexp::Integer(vec![Some(11)])
    );
    assert_eq!(
        final_runtime.take_output(),
        vec![OutputEvent::Stdout("11".into())]
    );
    Ok(())
}

#[test]
fn exported_selected_exiting_head_cannot_reactivate_expired_target() -> Result {
    let saved = {
        let mut origin = Frontend::default();
        origin.eval_source("saved<-NULL;tryCatch({saved<-.Internal(.addCondHands(NULL,NULL,NULL,NULL,FALSE));stop('FIRST')},error=function(e)e$message);saved")?
            .pop().ok_or("saved exiting head")?
    };
    let Sexp::PairListObject(head) = &saved else {
        panic!("ordinary exiting head")
    };
    let Sexp::ListObject(entry) = head.car()?.value else {
        panic!("ordinary entry")
    };
    assert!(matches!(entry.element(1)?, Sexp::Environment(_)));
    assert!(matches!(entry.element(3)?, Sexp::Environment(_)));
    let Sexp::ListObject(result) = entry.element(4)? else {
        panic!("shared result")
    };
    assert_eq!(result.element(0)?, Sexp::Nil);
    drop(result);
    drop(entry);
    let mut consumer = Frontend::default();
    consumer.eval_source("x<-99L")?;
    bind(consumer.runtime_mut(), "saved", saved)?;
    assert_eq!(consumer.eval_source("g<-function(){.Internal(.resetCondHands(saved));signalCondition(simpleError('SECOND'));99L};g()"), Err(EvalError::Control("no function to return from, jumping to top level".into())));
    assert_eq!(
        consumer.eval_source("sys.nframe();x")?,
        vec![Sexp::Integer(vec![Some(0)]), Sexp::Integer(vec![Some(99)])]
    );
    Ok(())
}

#[test]
fn sdk_fault_call_runtime_leaves_preserve_identity_in_both_handler_kinds() -> Result {
    let values = {
        let mut origin = Frontend::default();
        origin.eval_source("x<-11L;new.env();function()x")?
    };
    let environment = values[1].clone();
    let closure = values[2].clone();
    let expression = call(
        Sexp::Primitive(PrimitiveId::Nargs),
        vec![
            environment.clone(),
            closure.clone(),
            Sexp::Primitive(PrimitiveId::Quote),
        ],
    );
    drop(values);
    for wrapper in ["withCallingHandlers", "tryCatch"] {
        let mut receiver = Frontend::default();
        let handler = receiver
            .eval_source("seen<-NULL;bridge<-NULL;function(e){bridge<<-parent.frame();seen<<-conditionCall(e);conditionCall(e)}")?
            .pop()
            .ok_or("handler")?;
        let invocation = Sexp::Call {
            function: Box::new(Sexp::Symbol(wrapper.into())),
            arguments: vec![
                CallArgument::positional(expression.clone()),
                CallArgument {
                    name: Some("error".into()),
                    value: handler,
                },
            ],
        };
        let result = receiver.runtime_mut().eval(&invocation);
        if wrapper == "withCallingHandlers" {
            assert_eq!(
                result,
                Err(EvalError::Arity(
                    "3 arguments passed to 'nargs' which requires 0".into()
                ))
            );
        } else {
            assert_eq!(result?, expression);
        }
        if wrapper == "withCallingHandlers" {
            let Some(Sexp::Environment(bridge)) = receiver.runtime().get("bridge")? else {
                panic!("actual helper frame")
            };
            let Some(Sexp::Promise { state }) = bridge.get_local("call")? else {
                panic!("original call promise")
            };
            let quote = call(
                call(
                    Sexp::Symbol("::".into()),
                    vec![Sexp::Symbol("base".into()), Sexp::Symbol("quote".into())],
                ),
                vec![expression.clone()],
            );
            assert_eq!(state.expression()?, quote);
            let r_ust_sexp::Force::Cached(value) = state.begin_force()? else {
                panic!("completed call promise")
            };
            assert_eq!(value, expression);
        }
        let observed = receiver.runtime().get("seen")?.ok_or("observed call")?;
        assert_eq!(observed, expression);
        let Sexp::Call { arguments, .. } = observed else {
            panic!("canonical actual call")
        };
        assert_eq!(arguments[0].value, environment);
        assert_eq!(arguments[1].value, closure);
        assert_eq!(arguments[2].value, Sexp::Primitive(PrimitiveId::Quote));
        assert_eq!(
            receiver
                .runtime_mut()
                .eval(&call(arguments[1].value.clone(), vec![]))?,
            Sexp::Integer(vec![Some(11)])
        );
    }
    Ok(())
}

#[test]
fn unrepresentable_sdk_call_keeps_original_error_without_matching_handler() -> Result {
    let expression = call(
        Sexp::Primitive(PrimitiveId::Nargs),
        vec![Sexp::InternalCall {
            operation: InternalOperation::GetErrorMessage,
            arguments: vec![],
        }],
    );
    let mut receiver = Frontend::default();
    assert_eq!(
        receiver.runtime_mut().eval(&expression),
        Err(EvalError::Arity(
            "1 argument passed to 'nargs' which requires 0".into()
        ))
    );
    receiver.eval_source("seen<-FALSE;.Internal(.addCondHands('message',list(function(e)seen<<-TRUE),globalenv(),NULL,TRUE))")?;
    assert_eq!(
        receiver.runtime_mut().eval(&expression),
        Err(EvalError::Arity(
            "1 argument passed to 'nargs' which requires 0".into()
        ))
    );
    assert_eq!(
        receiver.runtime().get("seen")?,
        Some(Sexp::Logical(vec![Some(false)]))
    );
    Ok(())
}
