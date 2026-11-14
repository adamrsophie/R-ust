use r_ust_eval::{EvalError, RuntimeState};
use r_ust_sexp::{
    CallArgument, ClosureEnvironment, Environment, FunctionParameter, InternalOperation,
    ListElement, Sexp,
};

fn inspect(parameters: Vec<FunctionParameter>) -> Result<Sexp, EvalError> {
    RuntimeState::default().eval(&Sexp::InternalCall {
        operation: InternalOperation::Formals,
        arguments: vec![CallArgument::positional(Sexp::Closure {
            parameters,
            body: Box::new(Sexp::Nil),
            environment: ClosureEnvironment::Base,
        })],
    })
}

#[test]
fn formals_sdk_keeps_duplicate_names_order_and_missing_defaults_as_data() {
    assert_eq!(
        inspect(vec![
            FunctionParameter::required("same"),
            FunctionParameter::optional("same", Sexp::Integer(vec![Some(7)])),
        ])
        .unwrap(),
        Sexp::pairlist(vec![
            ListElement {
                name: Some("same".into()),
                value: Sexp::MissingValue
            },
            ListElement {
                name: Some("same".into()),
                value: Sexp::Integer(vec![Some(7)])
            },
        ])
    );
}

#[test]
fn formals_sdk_rejects_runtime_defaults_without_evaluating_them() {
    for (default, expected) in [
        (
            Sexp::Environment(Environment::global()),
            "formals runtime default",
        ),
        (
            Sexp::InternalCall {
                operation: InternalOperation::Formals,
                arguments: vec![],
            },
            "formals internal default",
        ),
    ] {
        assert_eq!(
            inspect(vec![FunctionParameter::optional("x", default)]),
            Err(EvalError::NotImplemented(expected))
        );
    }
}
