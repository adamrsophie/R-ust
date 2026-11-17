use super::*;

fn rejected(value: &ValueObject, attributes: &[ListElement], expected: GraphError) {
    let heap = value.0.heap().unwrap();
    let before = state(heap);
    assert_eq!(value.with_attributes(attributes), Err(expected));
    assert_eq!(state(heap), before);
}

#[test]
fn attribute_replace_invalid_metadata_publishes_no_valid_prefix() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let source = Sexp::AttributedAtomic {
        values: AtomicVector::Real(vec![1.0]),
        attributes: vec![attribute("note", text("original"))],
    };
    let value = ValueObject::import(&owner, &source, ValueImport::Evaluated)?;
    let cases = vec![
        vec![attribute("names", Sexp::String(vec![]))],
        vec![attribute("class", integer(1))],
        vec![attribute("class", text("factor"))],
        vec![attribute("class", Sexp::String(vec![]))],
        vec![attribute("dup", integer(1)), attribute("dup", integer(2))],
        vec![ListElement {
            name: None,
            value: integer(1),
        }],
        vec![attribute("", integer(1))],
        vec![attribute("dim", Sexp::Integer(vec![Some(2), Some(2)]))],
        vec![attribute("dim", Sexp::Integer(vec![None]))],
        vec![attribute("dim", Sexp::Integer(vec![Some(-1)]))],
        vec![attribute("dimnames", Sexp::Vector(vec![text("x")]))],
        vec![
            attribute("dim", integer(1)),
            attribute("dimnames", Sexp::Vector(vec![Sexp::String(vec![])])),
        ],
        vec![attribute("row.names", Sexp::Real(vec![1.0]))],
        vec![attribute("row.names", Sexp::Integer(vec![None, None]))],
    ];
    for mut attributes in cases {
        attributes.insert(0, attribute("prefix", vector(vec![integer(9)], vec![])));
        rejected(&value, &attributes, GraphError::WrongKind);
        assert_eq!(value.snapshot()?, source);
    }
    rejected(
        &value,
        &[attribute("bad\0name", integer(1))],
        GraphError::InvalidCharacter,
    );
    rejected(
        &value,
        &[attribute("note", text("bad\0text"))],
        GraphError::InvalidCharacter,
    );
    Ok(())
}

#[test]
fn attribute_replace_foreign_stale_wrong_type_and_unrooted_metadata_are_atomic() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(&owner, &integer(1), ValueImport::Evaluated)?;
    let child = ValueObject::import(&owner, &integer(7), ValueImport::Evaluated)?;
    let foreign = Arc::new(Heap::default()).session();
    let cases = [
        (Sexp::Environment(foreign), GraphError::ForeignHeap),
        (
            Sexp::Managed(ValueObject(child.0.clone(), SexpType::Real)),
            GraphError::WrongKind,
        ),
        (
            Sexp::Managed(ValueObject(Link::Edge(child.0.id()), SexpType::Integer)),
            GraphError::Unrooted,
        ),
    ];
    for (bad, error) in cases {
        rejected(
            &value,
            &[
                attribute("prefix", child.transport()?),
                attribute("bad", bad),
            ],
            error,
        );
        assert_eq!(child.semantic_reference_count()?, 0);
    }
    let doomed = ValueObject::import(&owner, &integer(9), ValueImport::Evaluated)?;
    let id = doomed.0.id();
    drop(doomed);
    let stale = ValueObject(
        Link::Root(Arc::new(RootToken {
            heap: heap.clone(),
            id,
            semantic_owner: false,
        })),
        SexpType::Integer,
    );
    rejected(
        &value,
        &[
            attribute("prefix", child.transport()?),
            attribute("bad", Sexp::Managed(stale)),
        ],
        GraphError::StaleNode,
    );
    assert_eq!(child.semantic_reference_count()?, 0);
    assert_eq!(value.snapshot()?, integer(1));
    Ok(())
}

#[test]
fn attribute_replace_invalid_source_handles_do_not_publish_metadata() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(&owner, &integer(1), ValueImport::Evaluated)?;
    let attributes = [attribute("note", vector(vec![integer(9)], vec![]))];
    let wrong = ValueObject(value.0.clone(), SexpType::Real);
    rejected(&wrong, &attributes, GraphError::WrongKind);
    let unrooted = ValueObject(Link::Edge(value.0.id()), SexpType::Integer);
    let before = state(&heap);
    assert_eq!(
        unrooted.with_attributes(&attributes),
        Err(GraphError::Unrooted)
    );
    assert_eq!(state(&heap), before);
    let id = value.0.id();
    drop(value);
    drop(wrong);
    let stale = ValueObject(
        Link::Root(Arc::new(RootToken {
            heap: heap.clone(),
            id,
            semantic_owner: false,
        })),
        SexpType::Integer,
    );
    rejected(&stale, &attributes, GraphError::StaleNode);
    Ok(())
}

#[test]
fn attribute_replace_unrepresented_graph_roles_and_matrix_fail_without_wrapping() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let list =
        crate::ListObject::from_value(&owner, &Sexp::List(vec![attribute("a", integer(1))]))?;
    let code = crate::BytecodeObject::new(
        &owner,
        crate::BytecodeParts {
            code: vec![12, 1],
            constants: vec![integer(1)],
            expression: integer(1),
        },
    )?;
    let pairlist = crate::PairListObject::cons(&owner, &attribute("a", integer(1)), None)?;
    let promise = crate::PromiseState::new(
        &Sexp::Symbol("absent".into()),
        &owner,
        crate::EvaluationOrigin::Global,
        true,
    )?;
    let raws = [
        Sexp::PairListObject(pairlist),
        Sexp::Promise { state: promise },
        Sexp::Environment(owner.clone()),
        Sexp::ListObject(list),
        Sexp::Bytecode(code),
        Sexp::Matrix {
            data: Box::new(integer(1)),
            nrow: 1,
            ncol: 1,
        },
        Sexp::Symbol("x".into()),
        Sexp::Nil,
    ];
    for raw in raws {
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        rejected(
            &value,
            &[attribute("note", text("metadata"))],
            GraphError::WrongKind,
        );
        rejected(&value, &[], GraphError::WrongKind);
        assert_eq!(value.snapshot()?, raw);
    }
    Ok(())
}

#[test]
fn attribute_replace_character_and_closure_reject_invalid_normalized_metadata() -> Result {
    let owner = Environment::global();
    let sources = [
        Sexp::AttributedCharacter {
            values: vec![Some("a".into())],
            attributes: vec![],
        },
        Sexp::Closure {
            parameters: vec![],
            body: Box::new(integer(1)),
            environment: ClosureEnvironment::Captured(owner.clone()),
            attributes: vec![],
        },
    ];
    for source in sources {
        let value = ValueObject::import(&owner, &source, ValueImport::Evaluated)?;
        for attrs in [
            vec![attribute("class", integer(1))],
            vec![attribute("note", integer(1)), attribute("note", integer(2))],
            vec![attribute("names", Sexp::String(vec![]))],
        ] {
            rejected(&value, &attrs, GraphError::WrongKind);
            assert_eq!(value.snapshot()?, source);
        }
    }
    Ok(())
}
