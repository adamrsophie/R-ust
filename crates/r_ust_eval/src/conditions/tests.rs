use super::*;

fn stack(owner: &Environment, token: &ListObject) -> PairListObject {
    let result = ordinary_list(
        owner,
        vec![
            Sexp::Nil,
            Sexp::Nil,
            Sexp::Nil,
            Sexp::ListObject(token.clone()),
        ],
    )
    .unwrap();
    let entry = ordinary_list(
        owner,
        vec![
            Sexp::Char(Some("error".into())),
            Sexp::Environment(owner.clone()),
            Sexp::Integer(vec![Some(17)]),
            Sexp::Environment(owner.clone()),
            Sexp::ListObject(result),
        ],
    )
    .unwrap();
    PairListObject::cons(
        owner,
        &ListElement {
            name: None,
            value: Sexp::ListObject(entry),
        },
        None,
    )
    .unwrap()
}

#[test]
fn nonancestor_reset_preserves_cons_identity_and_unwind_clears_shared_slots() {
    let owner = Environment::global();
    let token = ordinary_list(&owner, vec![Sexp::Nil]).unwrap();
    let a = stack(&owner, &token);
    let b = stack(&owner, &token);
    let a_entry = entry::Entry::read(a.car().unwrap().value).unwrap();
    let mut state = Conditions::default();
    for head in [&a, &b, &a] {
        state
            .reset(&owner, &Sexp::PairListObject(head.clone()))
            .unwrap();
        assert!(state.head.as_ref().unwrap().same_object(head));
    }
    state.unwind(&Some(b)).unwrap();
    assert!(matches!(
        a_entry.object.element(1).unwrap(),
        Sexp::Environment(_)
    ));
    state.reset(&owner, &Sexp::PairListObject(a)).unwrap();
    state.unwind(&None).unwrap();
    assert_eq!(a_entry.object.element(1).unwrap(), Sexp::Nil);
    assert_eq!(a_entry.object.element(3).unwrap(), Sexp::Nil);
    assert_eq!(
        a_entry.object.element(2).unwrap(),
        Sexp::Integer(vec![Some(17)])
    );
}

#[test]
fn rejected_stack_reset_does_not_publish() {
    let owner = Environment::global();
    let token = ordinary_list(&owner, vec![Sexp::Nil]).unwrap();
    let a = stack(&owner, &token);
    let mut state = Conditions {
        head: Some(a.clone()),
        ..Conditions::default()
    };
    let malformed = PairListObject::cons(
        &owner,
        &ListElement {
            name: None,
            value: Sexp::Nil,
        },
        None,
    )
    .unwrap();
    assert!(state
        .reset(&owner, &Sexp::PairListObject(malformed))
        .is_err());
    assert!(state.head.as_ref().unwrap().same_object(&a));
    let other = Environment::global();
    let token = ordinary_list(&other, vec![Sexp::Nil]).unwrap();
    let incoming = stack(&other, &token);
    state
        .reset(&owner, &Sexp::PairListObject(incoming.clone()))
        .unwrap();
    assert!(state.head.as_ref().unwrap().same_object(&incoming));
    let malformed = PairListObject::cons(
        &other,
        &ListElement {
            name: None,
            value: Sexp::Nil,
        },
        None,
    )
    .unwrap();
    assert!(state
        .reset(&owner, &Sexp::PairListObject(malformed))
        .is_err());
    assert!(state.head.as_ref().unwrap().same_object(&incoming));
}

#[test]
fn selection_pops_without_clearing_parent_or_target() {
    let owner = Environment::global();
    let token = ordinary_list(&owner, vec![Sexp::Nil]).unwrap();
    let head = stack(&owner, &token);
    let mut state = Conditions {
        head: Some(head),
        ..Conditions::default()
    };
    let selected = state.select(&[Some("error".into())]).unwrap().unwrap();
    assert!(state.head.is_none());
    assert!(matches!(
        selected.object.element(1).unwrap(),
        Sexp::Environment(_)
    ));
    assert!(matches!(
        selected.object.element(3).unwrap(),
        Sexp::Environment(_)
    ));
}

#[test]
fn fixup_requires_legitimate_token_identity_and_null_payload() {
    let owner = Environment::global();
    let token = ordinary_list(&owner, vec![Sexp::Nil]).unwrap();
    let fake = ordinary_list(&owner, vec![Sexp::Nil]).unwrap();
    for (result_token, initial, expected) in [
        (
            &token,
            Sexp::Nil,
            Sexp::String(vec![Some("ORIGINAL".into())]),
        ),
        (&fake, Sexp::Nil, Sexp::Nil),
        (
            &token,
            Sexp::Integer(vec![Some(42)]),
            Sexp::Integer(vec![Some(42)]),
        ),
    ] {
        let result = ordinary_list(
            &owner,
            vec![
                initial,
                Sexp::Nil,
                Sexp::Nil,
                Sexp::ListObject(result_token.clone()),
            ],
        )
        .unwrap();
        let jump = ConditionJump {
            target: EvaluationEnvironmentId(1),
            result: result.clone(),
            token: Some(token.clone()),
            message: Some("ORIGINAL".into()),
        };
        jump.fixup().unwrap();
        assert_eq!(result.element(0).unwrap(), expected);
    }
}

#[test]
fn condition_target_uses_live_local_contexts_not_retained_environment_identity() {
    use crate::dynamic_context::ReturnContext;
    use crate::{EvalFlow, RuntimeState};

    let mut runtime = RuntimeState::new(Default::default());
    let retained = runtime.global.child().unwrap();
    let entry = ReturnContext {
        execution: retained.clone(),
        sysparent: runtime.global.clone(),
        callable: Sexp::Nil,
        arguments: Vec::new(),
        call: Sexp::Nil,
        generic: false,
    };
    runtime
        .with_return_context(entry, |runtime| {
            let outer = runtime.control.condition_target(&retained)?;
            let foreign = RuntimeState::new(Default::default());
            assert_eq!(
                foreign.control.condition_target(&retained),
                Err(EvalError::Control(
                    "no function to return from, jumping to top level".into()
                ))
            );
            let nested = ReturnContext {
                execution: retained.clone(),
                sysparent: runtime.global.clone(),
                callable: Sexp::Nil,
                arguments: Vec::new(),
                call: Sexp::Nil,
                generic: false,
            };
            runtime.with_return_context(nested, |runtime| {
                let nearest = runtime.control.condition_target(&retained)?;
                assert_ne!(nearest, outer);
                Ok(EvalFlow::Value(Sexp::Nil))
            })?;
            assert_eq!(runtime.control.condition_target(&retained), Ok(outer));
            Ok(EvalFlow::Value(Sexp::Nil))
        })
        .unwrap();
    assert_eq!(
        runtime.control.condition_target(&retained),
        Err(EvalError::Control(
            "no function to return from, jumping to top level".into()
        ))
    );
}
