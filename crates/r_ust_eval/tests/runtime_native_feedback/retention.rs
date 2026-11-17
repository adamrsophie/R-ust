use super::*;
use r_ust_sexp::{EvaluationOrigin, Force, ListObject, PromiseState};

#[test]
fn typed_validation_accepts_repeated_handles_and_does_not_force_promises() -> Result {
    let owner = Environment::global();
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let child = ValueObject::import(&owner, &string("retained"), ValueImport::Evaluated)?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![child.transport()?, child.transport()?]),
        attributes: vec![attr(
            "note",
            Sexp::Promise {
                state: promise.clone(),
            },
        )],
    };
    let mut runtime = RuntimeState::default();
    for value in [raw.clone(), managed(&owner, &raw)?] {
        let result = runtime.eval(&value)?;
        assert_eq!(result, value);
    }
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn typed_metadata_validation_terminates_through_explicit_list_wrapper_cycles() -> Result {
    let owner = Environment::global();
    let list = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let wrapper = Sexp::Managed(ValueObject::import(
        &owner,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?);
    list.replace_shared(&[(0, wrapper.clone())])?;
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1)]),
        attributes: vec![attr("note", wrapper)],
    };
    let mut runtime = RuntimeState::default();
    for value in [raw.clone(), managed(&owner, &raw)?] {
        assert_eq!(runtime.eval(&value)?, value);
    }
    Ok(())
}

#[test]
fn special_character_storage_conversion_and_nested_names_preserve_metadata() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "v<-structure('a',names='first',note=7L);x<-1:3;attr(x,'names')<-v;identical(attr(x,'names'),structure(c('a',NA_character_,NA_character_),names=c('first','','')))")?;
    truth(&mut f, "x<-structure(c(1.5,2.5),names=c('a','b'),note=7L);y<-x;storage.mode(y)<-'integer';storage.mode(y)<-'character';storage.mode(y)<-'double';identical(attributes(x),attributes(y)) && identical(as.double(y),c(1,2)) && identical(as.double(x),c(1.5,2.5))")?;
    Ok(())
}
