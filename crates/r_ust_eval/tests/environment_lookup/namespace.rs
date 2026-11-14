use super::*;

#[test]
fn namespace_precedence_shared_apply_resolution_and_global_fallback() {
    let mut source = Frontend::default();
    source
        .eval_source("identity <- function(x) 91L; abs <- function(x) 92L; sentinel <- 31L")
        .unwrap();
    let namespace = root(source.runtime(), EnvironmentRole::BaseNamespace);
    let mut consumer = RuntimeState::default();
    for (code, expected) in [
        ("identity(2L)", integer(2)),
        ("abs(-2)", Sexp::Real(vec![2.0])),
        ("(abs)(-2)", Sexp::Real(vec![2.0])),
        ("sentinel", integer(31)),
        (
            "lapply(list(-2), abs)",
            Sexp::List(vec![r_ust_sexp::ListElement {
                name: None,
                value: Sexp::Real(vec![2.0]),
            }]),
        ),
    ] {
        assert_eq!(
            invoke(&mut consumer, &closure(&namespace, expression(code))).unwrap(),
            expected,
            "{code}"
        );
    }
    assert_eq!(
        invoke(
            &mut consumer,
            &closure(&namespace, expression("apply(matrix(-2), 1L, abs)"))
        )
        .unwrap(),
        Sexp::Real(vec![2.0])
    );
    let frame = root(source.runtime(), EnvironmentRole::Global)
        .child()
        .unwrap();
    frame.insert("identity".into(), &integer(3)).unwrap();
    assert_eq!(
        invoke(
            &mut consumer,
            &closure(&frame, expression("lapply(list(2L), identity)"))
        )
        .unwrap(),
        Sexp::List(vec![r_ust_sexp::ListElement {
            name: None,
            value: integer(91)
        }])
    );
}

#[test]
fn explicit_operator_and_parenthesis_bindings_are_called_before_intrinsics() {
    let global = Environment::global();
    let frame = global
        .session_root(EnvironmentRole::Empty)
        .unwrap()
        .child()
        .unwrap();
    let mut consumer = RuntimeState::default();
    let function = Sexp::Closure {
        parameters: vec![r_ust_sexp::FunctionParameter {
            name: "...".into(),
            default: None,
        }],
        body: Box::new(integer(23)),
        environment: ClosureEnvironment::Captured(frame.clone()),
    };
    for (name, code) in [
        ("!", "!1L"),
        ("<", "1L < 2L"),
        ("&&", "FALSE && TRUE"),
        ("%in%", "1L %in% 2L"),
        ("(", "(1L)"),
    ] {
        frame.insert(name.into(), &function).unwrap();
        assert_eq!(
            invoke(&mut consumer, &closure(&frame, expression(code))).unwrap(),
            integer(23)
        );
    }
}

#[test]
fn parenthesized_scalar_is_not_a_function_search() {
    let mut runtime = Frontend::default();
    runtime.eval_source("sum <- 1L").unwrap();
    for code in ["(sum)(2L)", "lapply(list(2L), (sum))"] {
        assert_eq!(
            runtime.eval_source(code),
            Err(EvalError::Type("attempt to apply non-function".into()))
        );
    }
    assert_eq!(
        runtime.eval_source("sum(2L)").unwrap().pop(),
        Some(integer(2))
    );
}
