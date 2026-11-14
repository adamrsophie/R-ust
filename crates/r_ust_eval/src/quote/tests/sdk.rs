use crate::{format_sexp, CallArgument, EvalError, Frontend, RuntimeState, Sexp};
use r_ust_sexp::{Environment, EvaluationOrigin, Force, PrimitiveId, PromiseState};

pub(super) fn quote_call(value: Sexp) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Quote)),
        arguments: vec![CallArgument::positional(value)],
    }
}

#[test]
fn raw_empty_tag_is_accepted_and_unsupported_printing_keeps_metadata_and_visibility() {
    assert_eq!(
        Frontend::default()
            .eval_source("(function(f) withVisible(f(x))$visible)(invisible(quote))")
            .unwrap(),
        vec![Sexp::Logical(vec![Some(true)])]
    );
    let mut runtime = RuntimeState::default();
    runtime
        .set_option("warnPartialMatchArgs", Sexp::Logical(vec![Some(true)]))
        .unwrap();
    runtime
        .set_option("warn", Sexp::Integer(vec![Some(2)]))
        .unwrap();
    let empty_tag = Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::Quote)),
        arguments: vec![CallArgument::named("", Sexp::Symbol("x".into()))],
    };
    assert_eq!(runtime.eval(&empty_tag).unwrap(), Sexp::Symbol("x".into()));
    for (value, kind, error) in [
        (
            Sexp::Symbol("é".into()),
            "symbol",
            EvalError::NotImplemented("language printer non-ASCII text profile"),
        ),
        (
            Sexp::Call {
                function: Box::new(Sexp::Symbol("if".into())),
                arguments: vec![],
            },
            "language",
            EvalError::Type("malformed canonical language call".into()),
        ),
    ] {
        runtime.visible = false;
        let quoted = runtime.eval(&quote_call(value.clone())).unwrap();
        assert_eq!(quoted, value);
        assert!(runtime.visible);
        assert_eq!(format_sexp(&quoted), Err(error));
        assert_eq!(crate::typeof_value(&quoted), kind);
        assert!(runtime.take_output().is_empty());
    }
}

#[test]
fn quote_runtime_handles_are_rejected_without_forcing() {
    let environment = Environment::global();
    let state = PromiseState::new(
        &Sexp::Symbol("unbound".into()),
        &environment,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&quote_call(Sexp::Promise {
            state: state.clone()
        })),
        Err(EvalError::NotImplemented("quoting runtime handles"))
    );
    assert!(matches!(state.begin_force().unwrap(), Force::Evaluate(_)));
    assert_eq!(
        runtime.eval(&quote_call(Sexp::InternalCall {
            operation: r_ust_sexp::InternalOperation::Vector,
            arguments: vec![],
        })),
        Err(EvalError::NotImplemented("quoting internal dispatch"))
    );
    assert!(runtime.take_output().is_empty());
}

#[test]
fn quote_computed_missing_promises_cache_across_export_and_origin_drop() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("n <- 0L; f <- function(x) { x; x; missing(x) }")
        .unwrap();
    assert_eq!(
        frontend
            .eval_source("f({n <<- n + 1L; quote(expr=)})")
            .unwrap(),
        vec![Sexp::Logical(vec![Some(false)])]
    );
    assert_eq!(
        frontend.eval_source("n").unwrap(),
        vec![Sexp::Integer(vec![Some(1)])]
    );
    for cached in [false, true] {
        let environment = Environment::global();
        let state = PromiseState::new(
            &quote_call(Sexp::MissingValue),
            &environment,
            EvaluationOrigin::Global,
            true,
        )
        .unwrap();
        let promise = Sexp::Promise {
            state: state.clone(),
        };
        if cached {
            assert_eq!(
                frontend.runtime_mut().eval(&promise).unwrap(),
                Sexp::MissingValue
            );
        }
        environment.insert("value".into(), &promise).unwrap();
        let exported = environment.get("value").unwrap().unwrap();
        drop(promise);
        drop(environment);
        let mut runtime = RuntimeState::default();
        assert_eq!(runtime.eval(&exported).unwrap(), Sexp::MissingValue);
        assert_eq!(runtime.eval(&exported).unwrap(), Sexp::MissingValue);
        assert!(state.supplied().unwrap());
        assert!(matches!(
            state.begin_force().unwrap(),
            Force::Cached(Sexp::MissingValue)
        ));
    }
}
