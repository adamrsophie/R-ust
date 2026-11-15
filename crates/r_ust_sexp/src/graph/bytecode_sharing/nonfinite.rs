use super::*;
use crate::ListElement;

fn nonfinite_closure(owner: &Environment, value: f64) -> Sexp {
    Sexp::Closure {
        parameters: vec![FunctionParameter {
            name: "x".into(),
            default: Some(Sexp::Real(vec![value])),
        }],
        body: Box::new(Sexp::Real(vec![value])),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![ListElement {
            name: Some("payload".into()),
            value: Sexp::Real(vec![value]),
        }],
    }
}

#[test]
fn identical_nonfinite_source_defaults_and_attributes_finalize() -> Result<(), GraphError> {
    for number in [
        crate::NA_REAL,
        f64::NAN,
        -0.0,
        0.0,
        f64::from_bits(0x7ff8_0000_0000_0001),
    ] {
        let heap = Arc::new(Heap::default());
        let global = heap.session();
        let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
        let original = nonfinite_closure(&namespace, number);
        global.install_base(&[("f".into(), original.clone())])?;
        let (replacement, bytecode) = compiled(&namespace, &original)?;
        global.finalize_base_bytecode(&[("f".into(), original, replacement.clone())])?;
        assert!(global.base_bytecode_ready()?);
        assert_eq!(references(&heap, bytecode.0.id())?, 1);
        let installed = namespace.get_local("f")?.ok_or(GraphError::WrongKind)?;
        assert!(super::super::super::values::same_snapshot(
            &installed,
            &replacement
        )?);
    }
    Ok(())
}

#[test]
fn changed_nonfinite_metadata_is_rejected_before_publication() -> Result<(), GraphError> {
    for position in 0..3 {
        let heap = Arc::new(Heap::default());
        let global = heap.session();
        let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
        let original = nonfinite_closure(&namespace, crate::NA_REAL);
        global.install_base(&[("f".into(), original.clone())])?;
        let (mut replacement, _) = compiled(&namespace, &original)?;
        let Sexp::Closure {
            parameters,
            body,
            attributes,
            ..
        } = &mut replacement
        else {
            return Err(GraphError::WrongKind);
        };
        match position {
            0 => parameters[0].default = Some(Sexp::Real(vec![f64::NAN])),
            1 => attributes[0].value = Sexp::Real(vec![f64::NAN]),
            _ => {
                *body = Box::new(Sexp::Bytecode(code(
                    &namespace,
                    Sexp::Real(vec![f64::NAN]),
                )?))
            }
        }
        assert_eq!(
            global.finalize_base_bytecode(&[("f".into(), original.clone(), replacement)]),
            Err(GraphError::InvalidBytecode)
        );
        assert!(!global.base_bytecode_ready()?);
        let installed = namespace.get_local("f")?.ok_or(GraphError::WrongKind)?;
        assert!(super::super::super::values::same_snapshot(
            &installed, &original
        )?);
    }
    Ok(())
}

#[test]
fn changed_current_binding_with_nonfinite_metadata_is_not_accepted() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let global = heap.session();
    let namespace = global.session_root(EnvironmentRole::BaseNamespace)?;
    let installed = nonfinite_closure(&namespace, crate::NA_REAL);
    global.install_base(&[("f".into(), installed.clone())])?;
    let previous = nonfinite_closure(&namespace, f64::NAN);
    let (replacement, _) = compiled(&namespace, &previous)?;
    assert_eq!(
        global.finalize_base_bytecode(&[("f".into(), previous, replacement)]),
        Err(GraphError::WrongKind)
    );
    assert!(!global.base_bytecode_ready()?);
    let actual = namespace.get_local("f")?.ok_or(GraphError::WrongKind)?;
    assert!(super::super::super::values::same_snapshot(
        &actual, &installed
    )?);
    Ok(())
}
