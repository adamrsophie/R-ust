use super::*;
use crate::{
    ClosureEnvironment, Force, FunctionParameter, ListElement, PromiseState, Sexp, ValueImport,
    ValueObject,
};

fn raw_closure(environment: ClosureEnvironment) -> Sexp {
    Sexp::Closure {
        parameters: vec![
            FunctionParameter::required("required"),
            FunctionParameter::optional("default", Sexp::List(vec![])),
        ],
        body: Box::new(Sexp::Call {
            function: Box::new(Sexp::Symbol("never_evaluate".into())),
            arguments: vec![],
        }),
        environment,
        attributes: vec![ListElement {
            name: Some("meta".into()),
            value: Sexp::Vector(vec![Sexp::Nil]),
        }],
    }
}

#[test]
fn aggregate_transport_borrowed_closure_preserves_typed_metadata_and_child_identity(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    for environment in [
        ClosureEnvironment::Base,
        ClosureEnvironment::Captured(owner.clone()),
    ] {
        let raw = raw_closure(environment);
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let children = value.view()?.children;
        let borrowed = value.borrow()?;
        assert!(borrowed.owner().same_object(&value));
        let Sexp::Closure {
            parameters,
            body,
            attributes,
            ..
        } = borrowed.value()
        else {
            panic!("closure projection")
        };
        assert_eq!(parameters[0], FunctionParameter::required("required"));
        assert_eq!(parameters[1].name, "default");
        let Some(Sexp::Managed(default)) = &parameters[1].default else {
            panic!("default identity")
        };
        assert!(default.same_object(&children[0].view()?.children[1]));
        let Sexp::Managed(body) = body.as_ref() else {
            panic!("body identity")
        };
        assert!(body.same_object(&children[1]));
        let Sexp::Managed(attribute) = &attributes[0].value else {
            panic!("attribute identity")
        };
        assert!(attribute.same_object(&children[3].view()?.children[0]));
        assert_eq!(borrowed.value().snapshot()?, raw);
        assert_eq!(owner.get_local("never_evaluate")?, None);
        assert_eq!(value.semantic_reference_count()?, 0);
        assert_eq!(body.semantic_reference_count()?, 1);
        drop(value);
        assert_eq!(borrowed.owner().snapshot()?, raw);
    }
    Ok(())
}

#[test]
fn aggregate_transport_projection_preserves_container_flags_names_and_dimensions(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let element = ListElement {
        name: Some("a".into()),
        value: Sexp::Integer(vec![Some(1)]),
    };
    for raw in [
        Sexp::Expression {
            elements: vec![element.clone()],
            names_present: true,
        },
        Sexp::NamedList {
            elements: vec![element.clone()],
            class: Some(Box::new(Sexp::String(vec![Some("custom".into())]))),
            class_first: false,
        },
        Sexp::ClassedList {
            elements: vec![element.clone()],
            class: Box::new(Sexp::String(vec![Some("custom".into())])),
            class_first: true,
        },
        Sexp::Matrix {
            data: Box::new(element.value.clone()),
            nrow: 1,
            ncol: 1,
        },
        Sexp::DataFrame {
            columns: vec![element.clone()],
            nrow: 1,
        },
        Sexp::pairlist(vec![element.clone()]),
        Sexp::Dots(vec![element]),
    ] {
        let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let borrow = object.borrow()?;
        assert_eq!(borrow.value().snapshot()?, raw);
        assert_eq!(object.semantic_reference_count()?, 0);
        for child in object.view()?.children {
            assert_eq!(child.semantic_reference_count()?, 1);
        }
    }
    Ok(())
}

#[test]
fn aggregate_transport_mixed_public_snapshot_materializes_children_without_forcing_explicit_handles(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(5)]),
        ValueImport::Evaluated,
    )?;
    let promise = PromiseState::new(
        &Sexp::Symbol("absent".into()),
        &owner,
        crate::EvaluationOrigin::Global,
        true,
    )?;
    let raw = Sexp::Closure {
        parameters: vec![FunctionParameter::optional("x", value.transport()?)],
        body: Box::new(value.transport()?),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![ListElement {
            name: Some("promise".into()),
            value: Sexp::Promise {
                state: promise.clone(),
            },
        }],
    };
    let Sexp::Closure {
        parameters,
        body,
        attributes,
        environment,
    } = raw.snapshot()?
    else {
        panic!("snapshot")
    };
    assert_eq!(parameters[0].default, Some(Sexp::Integer(vec![Some(5)])));
    assert_eq!(*body, Sexp::Integer(vec![Some(5)]));
    assert_eq!(environment, ClosureEnvironment::Captured(owner));
    assert_eq!(
        attributes[0].value,
        Sexp::Promise {
            state: promise.clone()
        }
    );
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    assert_eq!(value.semantic_reference_count()?, 0);
    Ok(())
}
