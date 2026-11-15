use r_ust_eval::{EvalError, RuntimeState};
use r_ust_sexp::{
    CallArgument, ClosureEnvironment, Environment, FunctionParameter, InternalOperation,
    ListElement, Sexp,
};

fn inspect(parameters: Vec<FunctionParameter>) -> Result<Sexp, EvalError> {
    RuntimeState::default().eval(&Sexp::InternalCall {
        operation: InternalOperation::Formals,
        arguments: vec![CallArgument::positional(Sexp::Closure {
            attributes: Vec::new(),
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
fn formals_sdk_preserves_runtime_defaults_without_evaluating_them() {
    let environment = Sexp::Environment(Environment::global());
    assert_eq!(
        inspect(vec![FunctionParameter::optional("x", environment.clone())]).unwrap(),
        Sexp::pairlist(vec![ListElement {
            name: Some("x".into()),
            value: environment
        }])
    );
    let internal = Sexp::InternalCall {
        operation: InternalOperation::Formals,
        arguments: vec![],
    };
    assert_eq!(
        inspect(vec![FunctionParameter::optional("x", internal)]),
        Err(EvalError::NotImplemented("formals internal default"))
    );
}
