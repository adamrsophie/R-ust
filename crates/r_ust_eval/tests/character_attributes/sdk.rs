use super::*;

pub(super) fn run() -> Result {
    run_kind(false)
}

pub(super) fn run_class() -> Result {
    run_kind(true)
}

fn run_kind(class_storage: bool) -> Result {
    let mut consumer = Frontend::default();
    let alias;
    {
        let mut origin = Frontend::default();
        let function = origin
            .eval_source("function() NULL")?
            .pop()
            .ok_or("closure missing")?;
        let Sexp::Closure {
            environment: ClosureEnvironment::Captured(global),
            ..
        } = function
        else {
            panic!("captured global");
        };
        let frame = global.child()?;
        frame.insert("x".into(), &Sexp::Integer(vec![Some(42)]))?;
        alias = frame.clone();
        let callback = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Environment(frame.clone())),
            environment: ClosureEnvironment::Captured(frame.clone()),
        };
        let value = attributed(
            &[Some("b")],
            vec![attribute(
                "custom",
                Sexp::List(vec![
                    attribute("environment", Sexp::Environment(frame)),
                    attribute("callback", callback),
                ]),
            )],
        );
        let value = if class_storage {
            Sexp::ClassedList {
                elements: vec![],
                class: Box::new(value),
                class_first: true,
            }
        } else {
            value
        };
        bind(origin.runtime_mut(), "bundle", value)?;
        let exported = origin.runtime().get("bundle")?.ok_or("export missing")?;
        bind(consumer.runtime_mut(), "retained", exported)?;
    }
    let expression = if class_storage {
        "class(retained)"
    } else {
        "retained"
    };
    consumer.eval_source(&format!("x<-99L; payload<-attr({expression},'custom'); callback<-payload$callback; observed<-callback()"))?;
    let observed = consumer
        .runtime()
        .get("observed")?
        .ok_or("observed missing")?;
    assert_eq!(observed, Sexp::Environment(alias.clone()));
    alias.insert("x".into(), &Sexp::Integer(vec![Some(73)]))?;
    truth(
        &mut consumer,
        "payload$environment$x==73L && observed$x==73L && x==99L",
    )?;
    let final_value = consumer
        .runtime()
        .get("retained")?
        .ok_or("retained missing")?;
    drop(consumer);
    let character = match &final_value {
        Sexp::ClassedList { class, .. } => class.as_ref(),
        other => other,
    };
    let Sexp::AttributedCharacter { attributes, .. } = character else {
        panic!("lost attrs");
    };
    let Sexp::List(payload) = &attributes[0].value else {
        panic!("lost list");
    };
    assert_eq!(payload[0].value, observed);
    drop(observed);
    drop(alias);
    drop(final_value);
    Ok(())
}
