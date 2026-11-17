use super::*;

fn payload(numbers: [f64; 3]) -> Sexp {
    Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![numbers[0]]),
            attributes: vec![attribute("inside", Sexp::Real(vec![numbers[1]]))],
        }]),
        attributes: vec![attribute("outside", Sexp::Real(vec![numbers[2]]))],
    }
}

fn closure(owner: &Environment, numbers: [f64; 3]) -> Sexp {
    Sexp::Closure {
        parameters: vec![FunctionParameter::optional("x", payload(numbers))],
        body: Box::new(integer(1)),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![attribute("metadata", payload(numbers))],
    }
}

fn compiled(owner: &Environment, value: &Sexp) -> std::result::Result<Sexp, GraphError> {
    let mut value = value.clone();
    let Sexp::Closure { body, .. } = &mut value else {
        return Err(GraphError::WrongKind);
    };
    *body = Box::new(Sexp::Bytecode(code(owner, *body.clone())?));
    Ok(value)
}

#[test]
fn managed_composition_nonfinite_typed_metadata_finalizes_only_with_exact_bits() -> Result {
    for number in [
        crate::NA_REAL,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0001),
        -0.0,
        0.0,
    ] {
        for changed in [None, Some(0), Some(1), Some(2)] {
            let heap = Arc::new(Heap::default());
            let owner = heap.session();
            let namespace = owner.session_root(crate::EnvironmentRole::BaseNamespace)?;
            let original = closure(&namespace, [number; 3]);
            owner.install_base(&[("f".into(), original.clone())])?;
            let mut replacement = compiled(&namespace, &original)?;
            if let Some(index) = changed {
                let mut numbers = [number; 3];
                numbers[index] = f64::from_bits(number.to_bits() ^ 1);
                let Sexp::Closure { attributes, .. } = &mut replacement else {
                    panic!("closure")
                };
                attributes[0].value = payload(numbers);
            }
            let result = owner.finalize_base_bytecode(&[(
                "f".into(),
                original.clone(),
                replacement.clone(),
            )]);
            let expected = if changed.is_some() {
                Err(GraphError::InvalidBytecode)
            } else {
                Ok(())
            };
            assert_eq!(result, expected);
            assert_eq!(owner.base_bytecode_ready()?, changed.is_none());
            let installed = namespace.get_local("f")?.ok_or(GraphError::WrongKind)?;
            assert!(crate::graph::values::same_snapshot(
                &installed,
                if changed.is_some() {
                    &original
                } else {
                    &replacement
                },
            )?);
        }
    }
    Ok(())
}

#[test]
fn managed_composition_changed_current_typed_nan_metadata_rejects_before_publication() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let namespace = owner.session_root(crate::EnvironmentRole::BaseNamespace)?;
    let installed = closure(&namespace, [crate::NA_REAL; 3]);
    owner.install_base(&[("f".into(), installed.clone())])?;
    let previous = closure(&namespace, [f64::NAN; 3]);
    let replacement = compiled(&namespace, &previous)?;
    assert_eq!(
        owner.finalize_base_bytecode(&[("f".into(), previous, replacement)]),
        Err(GraphError::WrongKind)
    );
    assert!(!owner.base_bytecode_ready()?);
    let actual = namespace.get_local("f")?.ok_or(GraphError::WrongKind)?;
    assert!(crate::graph::values::same_snapshot(&actual, &installed)?);
    Ok(())
}
