use r_ust_eval::{format_sexp, EvalError, Frontend, RuntimeState};
use r_ust_sexp::{
    AssignmentOperator, CallArgument, ClosureEnvironment, EvaluationOrigin, Force, ListElement,
    PromiseState, Sexp,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}

fn truth(frontend: &mut Frontend, source: &str) -> Result {
    let result = frontend.eval_source(source)?;
    assert_eq!(
        result.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}"
    );
    Ok(())
}

#[test]
fn ordinary_attributes_retain_replace_remove_and_clone() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function(x=stop('forced')) x; g<-f; attr(f,'srcref')<-17L")?;
    truth(
        &mut frontend,
        "identical(attr(f,'srcref'),17L) && identical(f,g) && !identical(f,g,ignore.srcref=FALSE)",
    )?;
    frontend.eval_source("attr(f,'srcref')<-23L")?;
    truth(
        &mut frontend,
        "identical(attr(f,'srcref'),23L) && is.null(attr(g,'srcref'))",
    )?;
    frontend.eval_source(
        "attr(f,'srcref')<-NULL; attr(f,'payload')<-list(2L,quote(stop('unforced')))",
    )?;
    truth(&mut frontend, "!identical(f,g) && typeof(f)=='closure' && mode(f)=='function' && class(f)=='function' && identical(environment(f),environment(g))")?;
    frontend.eval_source("attributes(f)<-list(payload=1L,srcref=12L,payload=3L)")?;
    truth(
        &mut frontend,
        "identical(attributes(f),list(payload=3L,srcref=12L)) && identical(attr(f,'pay'),3L)",
    )?;
    frontend.eval_source("attributes(f)<-NULL")?;
    truth(
        &mut frontend,
        "is.null(attributes(f)) && identical(f,g,ignore.srcref=FALSE)",
    )?;
    Ok(())
}

#[test]
fn closure_class_preserves_callability_and_dispatch() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function(x) x+1L; class(f)<-'custom'")?;
    truth(&mut frontend, "class(f)=='custom' && is.object(f) && inherits(f,'custom') && typeof(f)=='closure' && mode(f)=='function' && identical(f(2L),3L)")?;
    frontend.eval_source("g<-function(x) UseMethod('g'); g.custom<-function(x) 42L")?;
    truth(&mut frontend, "identical(g(f),42L)")?;
    frontend.eval_source("class(f)<-NULL")?;
    truth(
        &mut frontend,
        "class(f)=='function' && !is.object(f) && identical(f(2L),3L)",
    )?;
    Ok(())
}

#[test]
fn attributes_identity_order_and_default_source_ignoring() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function() 1L; g<-f; attr(f,'a')<-1L; attr(f,'b')<-2L; attr(g,'b')<-2L; attr(g,'a')<-1L; attr(f,'srcref')<-9L")?;
    truth(&mut frontend, "identical(f,g) && !identical(f,g,attrib.as.set=FALSE) && !identical(f,g,ignore.srcref=FALSE)")?;
    Ok(())
}

#[test]
fn integer_source_and_ordinary_attributes_have_reference_print_text() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function(x) x; attr(f,'srcref')<-17L")?;
    assert_eq!(
        format_sexp(&frontend.runtime().get("f")?.ok_or("f")?)?,
        "17"
    );
    frontend.eval_source(
        "attr(f,'srcref')<-NULL; attr(f,'payload')<-list(2L,quote(stop('unforced')))",
    )?;
    assert_eq!(format_sexp(&frontend.runtime().get("f")?.ok_or("f")?)?, "function (x) \nx\nattr(,\"payload\")\nattr(,\"payload\")[[1]]\n[1] 2\n\nattr(,\"payload\")[[2]]\nstop(\"unforced\")\n");
    frontend.eval_source("attr(f,'srcref')<-quote(stop('source forced'))")?;
    assert_eq!(
        format_sexp(&frontend.runtime().get("f")?.ok_or("f")?),
        Err(EvalError::NotImplemented(
            "closure source-reference print dispatch"
        ))
    );
    Ok(())
}

#[test]
fn nested_attribute_roots_survive_origin_and_consumer_without_forcing() -> Result {
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
            panic!("global")
        };
        let frame = global.child()?;
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        let pending = PromiseState::new(
            &Sexp::Symbol("never_force".into()),
            &frame,
            EvaluationOrigin::Global,
            true,
        )?;
        let callback = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Symbol("marker".into())),
            environment: ClosureEnvironment::Captured(frame.clone()),
            attributes: vec![],
        };
        let value = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Nil),
            environment: ClosureEnvironment::Base,
            attributes: vec![ListElement {
                name: Some("payload".into()),
                value: Sexp::List(vec![
                    ListElement {
                        name: Some("environment".into()),
                        value: Sexp::Environment(frame),
                    },
                    ListElement {
                        name: Some("callback".into()),
                        value: callback,
                    },
                    ListElement {
                        name: Some("pending".into()),
                        value: Sexp::Promise { state: pending },
                    },
                ]),
            }],
        };
        bind(origin.runtime_mut(), "bundle", value)?;
        origin.runtime().get("bundle")?.ok_or("bundle missing")?
    };
    let mut consumer = Frontend::default();
    bind(consumer.runtime_mut(), "retained", exported.clone())?;
    consumer.eval_source(
        "marker<-99L; payload<-attr(retained,'payload'); callback<-payload$callback",
    )?;
    truth(
        &mut consumer,
        "identical(callback(),42L) && identical(marker,99L) && identical(retained,retained)",
    )?;
    let retained = consumer.runtime().get("retained")?.ok_or("retained")?;
    drop(exported);
    drop(consumer);
    let Sexp::Closure { attributes, .. } = &retained else {
        panic!("closure")
    };
    let Sexp::List(payload) = &attributes[0].value else {
        panic!("payload")
    };
    let Sexp::Environment(frame) = &payload[0].value else {
        panic!("frame")
    };
    assert_eq!(
        frame.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    let Sexp::Promise { state } = &payload[2].value else {
        panic!("promise")
    };
    let Force::Evaluate(lease) = state.begin_force()? else {
        panic!("forced")
    };
    assert!(!lease.interrupted);
    assert_eq!(&lease.environment, frame);
    lease.complete(&Sexp::Nil)?;
    let mut receiver = RuntimeState::default();
    assert_eq!(
        receiver.eval(&Sexp::Call {
            function: Box::new(payload[1].value.clone()),
            arguments: Vec::<CallArgument>::new()
        })?,
        Sexp::Integer(vec![Some(42)])
    );
    Ok(())
}
