use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{
    AssignmentOperator, CallArgument, ClosureEnvironment, Environment, ListElement, ListObject,
    PairListObject, Sexp, SexpType,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn list(owner: &Environment, values: Vec<Sexp>) -> Result<ListObject> {
    Ok(ListObject::from_value(
        owner,
        &Sexp::List(
            values
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
        ),
    )?)
}

#[test]
fn exported_heads_and_result_token() -> Result {
    let producer = Environment::global();
    let consumer = Environment::global();
    let token = list(&producer, vec![Sexp::Nil])?;
    let fake = list(&producer, vec![Sexp::Nil])?;
    assert!(!token.same_object(&fake));
    let result = list(
        &producer,
        vec![
            Sexp::Nil,
            Sexp::Nil,
            Sexp::Nil,
            Sexp::ListObject(token.clone()),
        ],
    )?;
    let entry = list(
        &producer,
        vec![
            Sexp::Char(Some("error".into())),
            Sexp::Nil,
            Sexp::Closure {
                parameters: vec![],
                body: Box::new(Sexp::Integer(vec![Some(3)])),
                environment: ClosureEnvironment::Captured(producer.clone()),
            },
            Sexp::Nil,
            Sexp::ListObject(result.clone()),
        ],
    )?;
    entry.set_gp_shared(1)?;
    let exiting = entry.copy_with(&entry.value()?)?;
    exiting.set_gp_shared(0)?;
    assert_eq!(entry.header()?.gp(), 1);
    assert_eq!(exiting.header()?.gp(), 0);
    let car = ListElement {
        name: None,
        value: Sexp::ListObject(entry.clone()),
    };
    let a = PairListObject::cons(&producer, &car, None)?;
    let b = PairListObject::cons(&producer, &car, None)?;
    assert!(!a.same_object(&b));
    for head in [&a, &b, &a] {
        consumer.insert("head".into(), &Sexp::PairListObject(head.clone()))?;
        let Some(Sexp::PairListObject(saved)) = consumer.get_local("head")? else {
            panic!("head")
        };
        assert!(saved.same_object(head));
    }
    drop(producer);
    let Some(Sexp::PairListObject(saved)) = consumer.get_local("head")? else {
        panic!("head")
    };
    assert!(saved.same_object(&a));
    let Sexp::ListObject(saved_entry) = saved.car()?.value else {
        panic!("entry")
    };
    assert!(saved_entry.same_object(&entry));
    assert_eq!(Sexp::PairListObject(saved).sexptype(), SexpType::List);
    assert_eq!(
        Sexp::ListObject(saved_entry.clone()).sexptype(),
        SexpType::Vector
    );
    assert_eq!(saved_entry.element(0)?.sexptype(), SexpType::Char);
    entry.replace_shared(&[
        (1, Sexp::Integer(vec![Some(9)])),
        (3, Sexp::Integer(vec![Some(9)])),
    ])?;
    entry.replace_shared(&[(1, Sexp::Nil), (3, Sexp::Nil)])?;
    assert_eq!(saved_entry.element(1)?, Sexp::Nil);
    assert_eq!(saved_entry.element(3)?, Sexp::Nil);
    let Sexp::ListObject(saved_result) = saved_entry.element(4)? else {
        panic!("result")
    };
    assert!(saved_result.same_object(&result));
    let Sexp::ListObject(saved_token) = saved_result.element(3)? else {
        panic!("token")
    };
    assert!(saved_token.same_object(&token));
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("identical".into())),
            arguments: vec![
                CallArgument::positional(Sexp::ListObject(token)),
                CallArgument::positional(Sexp::ListObject(fake))
            ],
        })?,
        Sexp::Logical(vec![Some(true)])
    );
    Ok(())
}

#[test]
fn exported_head_outlives_origin_and_consumer_runtimes() -> Result {
    let mut consumer = Frontend::default();
    consumer.eval_source("marker<-99L")?;
    {
        let mut origin = Frontend::default();
        let values = origin.eval_source("marker<-11L;function()marker;globalenv()")?;
        let [_, callback, Sexp::Environment(owner)] = values.as_slice() else {
            panic!("origin callback and environment")
        };
        let token = list(owner, vec![Sexp::Nil])?;
        let result = list(
            owner,
            vec![Sexp::Nil, Sexp::Nil, Sexp::Nil, Sexp::ListObject(token)],
        )?;
        let entry = list(
            owner,
            vec![
                Sexp::Char(Some("error".into())),
                Sexp::Nil,
                callback.clone(),
                Sexp::Nil,
                Sexp::ListObject(result),
            ],
        )?;
        let head = PairListObject::cons(
            owner,
            &ListElement {
                name: None,
                value: Sexp::ListObject(entry),
            },
            None,
        )?;
        consumer.runtime_mut().eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: "saved".into(),
            value: Box::new(Sexp::PairListObject(head)),
        })?;
    }
    assert_eq!(
        consumer.eval_source("saved[[1L]][[3L]]()")?,
        vec![Sexp::Integer(vec![Some(11)])]
    );
    assert_eq!(
        consumer.eval_source("marker")?,
        vec![Sexp::Integer(vec![Some(99)])]
    );
    let Some(Sexp::PairListObject(retained)) = consumer.runtime().get("saved")? else {
        panic!("consumer exported head")
    };
    drop(consumer);
    let Sexp::ListObject(entry) = retained.car()?.value else {
        panic!("retained entry")
    };
    assert_eq!(entry.element(0)?, Sexp::Char(Some("error".into())));
    let mut final_runtime = RuntimeState::default();
    assert_eq!(
        final_runtime.eval(&Sexp::Call {
            function: Box::new(entry.element(2)?),
            arguments: vec![],
        })?,
        Sexp::Integer(vec![Some(11)])
    );
    drop(final_runtime);
    drop(entry);
    drop(retained);
    Ok(())
}
