use r_ust_eval::{format_sexp, EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{
    canonical_language, AssignmentOperator, CallArgument, ClosureEnvironment, LanguageError,
    ListElement, Sexp, SexpType,
};
#[path = "character_attributes/class_rhs.rs"]
mod class_rhs;
#[path = "character_attributes/consumers.rs"]
mod consumers;
#[path = "character_attributes/frozen.rs"]
mod frozen;
#[path = "character_attributes/member_order.rs"]
mod member_order;
#[path = "character_attributes/ordinary.rs"]
mod ordinary;
#[path = "character_attributes/sdk.rs"]
mod sdk;
#[path = "character_attributes/substring.rs"]
mod substring;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn strings(values: &[Option<&str>]) -> Sexp {
    Sexp::String(
        values
            .iter()
            .map(|value| value.map(str::to_string))
            .collect(),
    )
}
fn attributed(values: &[Option<&str>], attributes: Vec<ListElement>) -> Sexp {
    Sexp::AttributedCharacter {
        values: values
            .iter()
            .map(|value| value.map(str::to_string))
            .collect(),
        attributes,
    }
}
fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}
fn truth(frontend: &mut Frontend, source: &str) -> Result {
    let result = frontend.eval_source(source)?;
    assert_eq!(
        result.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}: {result:?}"
    );
    Ok(())
}
fn stdout(frontend: &mut Frontend) -> String {
    frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .filter_map(|event| match event {
            OutputEvent::Stdout(text) => Some(text),
            _ => None,
        })
        .collect()
}

#[test]
fn old_sdk_and_canonical_boundary() -> Result {
    let mut empty = Frontend::default();
    bind(
        empty.runtime_mut(),
        "x",
        attributed(&[Some("b"), None], vec![]),
    )?;
    truth(
        &mut empty,
        "is.vector(x) && identical(x,c('b',NA_character_)) && identical(c('b',NA_character_),x)",
    )?;
    let old = strings(&[Some("b"), None]);
    let value = attributed(
        &[Some("b"), None],
        vec![attribute("custom", Sexp::Integer(vec![Some(7)]))],
    );
    for input in [&old, &value] {
        assert_eq!(input.sexptype(), SexpType::String);
        assert_eq!(input.character_values(), old.character_values());
        let mut frontend = Frontend::default();
        bind(frontend.runtime_mut(), "x", input.clone())?;
        truth(&mut frontend, "is.character(x) && is.atomic(x) && typeof(x)=='character' && mode(x)=='character' && length(x)==2L")?;
    }
    assert_eq!(canonical_language(&old)?, old);
    assert_eq!(canonical_language(&value), Err(LanguageError::RuntimeValue));
    let call = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![CallArgument::positional(value.clone())],
    };
    assert_eq!(canonical_language(&call), Err(LanguageError::RuntimeValue));
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), "x", value.clone())?;
    assert_eq!(frontend.runtime().get("x")?, Some(value));
    Ok(())
}

#[test]
fn ordinary_attributes_and_validation() -> Result {
    ordinary::run()
}
#[test]
fn consumers_and_display() -> Result {
    consumers::run()
}
#[test]
fn sdk_retention_across_runtimes() -> Result {
    sdk::run()
}
